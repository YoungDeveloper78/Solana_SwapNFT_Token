import { buildAndSendTx, createAndFundATA, createFundedWallet, createTokenAuthorizationRules } from "../utils/pnft";
import { PNftTransferClient } from "../utils/PNftTransferClient";
import * as anchor from "@coral-xyz/anchor";
import {Program} from "@coral-xyz/anchor";
import { BN  } from "bn.js";
import { Neptune, IDL } from "../target/types/neptune"
// import { metadata } from "../target/idl/joker_swap.json";
import { getMerkleProof, getMerkleRoot, mintFromCandyMachineBuilder } from '@metaplex-foundation/js';
import { readFileSync } from "fs";
// import {Decimal} from "decimal.js";
import { PublicKey, Commitment, Keypair, Transaction,SystemProgram ,Connection, sendAndConfirmTransaction} from "@solana/web3.js"
import { ASSOCIATED_TOKEN_PROGRAM_ID , createAssociatedTokenAccountInstruction,TOKEN_2022_PROGRAM_ID as tokenProgram, createMint, createAccount, mintTo, getAssociatedTokenAddress, createAssociatedTokenAccount,TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount } from "@solana/spl-token"
import * as solanaSPLLib from "@solana/spl-token";
// import calculateValue from "./utils/calculateValue";

const commitment: Commitment = "confirmed"; // processed, confirmed, finalized

let connection = anchor.AnchorProvider.env().connection;
let nfts:{ mint: PublicKey, ata: PublicKey }[];
let pnft_mint_addr: PublicKey;
let pnft_ata_addr: PublicKey;

const provider = anchor.AnchorProvider.local("https://api.devnet.solana.com");
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.Neptune as Program<Neptune>;


let walletData = JSON.parse(readFileSync("~/.config/solana/id.json").toString());
const privateKey = Buffer.from(walletData); // TODO: needed ?
const wallet = Keypair.fromSecretKey(privateKey);

let id_num:number = 1; // this is the id of the pool which is initialied in the "init_pool" instruction.
const id = new Uint8Array(2);
id.set([id_num], 0);

// let wallet=new Keypair()
let token_mint:PublicKey; // this is the mint address of the Token22 that would be passed into instructions.
let TokenAccountPDA:PublicKey; // this is the PDA "hybrid_token_account" in contract would be the token in the pool that would be swapped with nfts.
let TokenAccountPDABump:number; // this is the Bump for PDA "hybrid_token_account" in contract.
let token_ata:PublicKey; // this is the assiciated token account that was created to hold token22 supply and its authority is minter which here would be the "wallet" variable.
let root:Buffer; // this is the root of merkle tree that was generated from list of all nfts that are going to be added to liquidity.

// this is the "pool_account" PDA initialized in initalize_pool intruction in contract
const [PoolAccountPDA, PoolTokenAccountBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("pool"), wallet.publicKey.toBuffer(),id], program.programId
);

const [UserData,  UserDataAccountBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("user-data"),wallet.publicKey.toBuffer(),PoolAccountPDA.toBuffer()], program.programId
);
// this is the PDA that holds the data for init_config struct(config) data which is created in the "init_config" instrunction.
const [Config,  ConfigBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("config")], program.programId
);

let nft_mints:Buffer[] = [];

// it("Airdrop", async () => {
//   this unit test can be used to fund the wallet address that is going to be paying for all the calls in the blockchain.
//   await Promise.all([wallet, wallet].map(async (k) => {
//     return await anchor.getProvider().connection.requestAirdrop(k.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL)
//   })).then(confirmTxs);
// });

// it("Mint nfts", async () => {
//   nfts=await mintNFTs(connection,wallet,10,TOKEN_2022_PROGRAM_ID)
// });

it("Deployment Test", async () => {
  console.log(await program.methods.test().accounts({}).rpc());
  // console.log(tx)
});

it("create token", async ()=>{
  // in this unit test we are creating the token (token22) which is going to be used in the pool.
  let {mint, ata} = (await newMintToAta(connection, wallet, 9, 21e9, TOKEN_2022_PROGRAM_ID));
  token_mint = mint; // mint address of the token22 that was minted.
  token_ata = ata; // token assiciated account address that holds supply of the mint address.
  
  // here we are getting token PDA that was created for token to hold its value which in contract its named : "hybrid_token_account"
  // this PDA holds (token22) in the pool which would be controlled by authority which would be "pool_account" in contract
  const [tokenPDA, tokenBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("token-pool"),
    PoolAccountPDA.toBuffer(), 
    token_mint.toBuffer()],
    program.programId
  );

  TokenAccountPDA = tokenPDA;
  TokenAccountPDABump = tokenBump;

  console.log("rewardToken", mint);
  console.log("ownerRewardTokenATA", ata);  
});

it("create merkle tree", async ()=>{
  // here we generate all 5 share holders. then we will pass all of these into "init_config" instruction and the will be saved in config data account.
  const creators = Array(5)
    .fill(null)
    .map((_) => ({ address: Keypair.generate().publicKey, share: 20 }));

  const { mint, ata } = await createAndFundATA({
    provider: provider,
    owner: wallet,
    creators,
    royaltyBps: 1000,
    programmable: true,
  });

  pnft_mint_addr = mint;
  pnft_ata_addr = ata;
  // we are making a list of all nfts that are going to be in the pool to create a root(merkle root) from their addresses.
  const nfts = [
      pnft_mint_addr.toString(),
      "Ehp1XDkvJ7vhtsibhDFH4pujxWqhFxyeSxTNemwXGDQM",
      "4obWbWpabe6EsGQbLzz5xFYyUKfa3HCyhmRvxRFJuaVp",
      "5Dhku7FcjEDDJEu9ngjVQ8LqKY5pjXSc3E94yCeeiDKU",
      "5NWg6nwiYH7Hiu6DtCfzTEGieUgdbPTRUhJVKp3VjPsC",
      "6EcFnAnkniBu4CYWd72oVKPvgHQGzVH3tjdD4uH2T7UP",
      "8MSZ7gRJxVHpaq7u8WwRgR4ZDZYEffNVNccurRqfPPbg",
      "8qbpJfMpEJH4ksfkkoDqnKedxkRzUVEBeX9QPHDfymLB",
      "9fFA6g1iWwwRPWXhTEAsPHR2ZJGbz8Cq1gehRDFi3uds",
  ];
  // from their list of strings we create a list of Public key objests to be paased to getMerkleRoot() method which would get array of Public keys.
  nft_mints = nfts.map((mint)=>{
    return new PublicKey(mint).toBuffer();
  });
  // we are creating the root to be passed into the init_pool to be saved into hybird pool data account we use root to know what nfts are going to be in the pool as a verification.
  root = Buffer.from(getMerkleRoot(nft_mints));
});

it("Initialize config", async () => {
  try {
    const tx = await program.methods.initConfig(ConfigBump, new BN(100), new BN(100), [wallet.publicKey,wallet.publicKey,wallet.publicKey,wallet.publicKey,wallet.publicKey], [20,20,20,20,20])
      .accounts({
        config:Config,
        signer:wallet.publicKey
      }).signers([wallet]).rpc();

    await confirmTx(tx);

  } catch(e) {
    console.error(e);
  }
});

it("initializ pool!", async () => {  
  const allowList2 = nft_mints // TODO: whats allow list 2? this might not be needed and can be removed.
  // let root=getMerkleRoot(allowList2)
  
  const txid0= await program.methods.initializePool(id_num,new BN(1000),Array.from(root))
    .accounts({
      poolAccount:PoolAccountPDA,
      tokenMint:token_mint,
      hybridTokenAccount:TokenAccountPDA,
      signer: wallet.publicKey,
      tokenProgram:TOKEN_2022_PROGRAM_ID,
      config:Config,
      feeReceiver1:wallet.publicKey,
      feeReceiver2:wallet.publicKey,
      feeReceiver3:wallet.publicKey,
      feeReceiver4:wallet.publicKey,
      feeReceiver5:wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([wallet]).rpc({skipPreflight: false}); 
  console.log(txid0);
});

it("add liquidity!", async () => {
  // console.log(await program.account.hybridPool.fetch(PoolAccountPDA))
  const pNftTransferClient = new PNftTransferClient(provider.connection, provider.wallet as anchor.Wallet,IDL,program.programId)
  const allowList2 = nft_mints

  let proof2=getMerkleProof(allowList2,pnft_mint_addr.toBuffer())

  const [nftPda,  nft_bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft-pool"),PoolAccountPDA.toBuffer(),pnft_mint_addr.toBuffer()], program.programId
  );
  // const nftPDAAccount = await getOrCreateAssociatedTokenAccount(
  //   provider.connection,
  //   wallet,
  //   token_mint,
  //   nftPda,
  //   true
  // );

  root=Buffer.from(getMerkleRoot([token_mint.toBuffer()]))
  const associatedTokenAccount = await getAssociatedTokenAddress(
    pnft_mint_addr,
    PoolAccountPDA,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
    const [nftPDA, tokenBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("token-pda-account"), PoolAccountPDA.toBuffer(), associatedTokenAccount.toBuffer()], program.programId)
      const PnftAssiciateTokenAccount = await createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        associatedTokenAccount,
        PoolAccountPDA,
        pnft_mint_addr,
      );

      const transaction = new Transaction().add(PnftAssiciateTokenAccount);

      // Send the transaction
      await sendAndConfirmTransaction(connection, transaction, [wallet]);
          
            const builder = await pNftTransferClient.addLiquidity({
    nftMint: pnft_mint_addr,
    userAta:pnft_ata_addr,
    poolAta: nftPDA,
    owner: wallet.publicKey,
    pool:PoolAccountPDA,
    id:id_num,
    proof:proof2,
    config:Config,
    userData: UserData,
    tokenMint: token_mint,
    hybridTokenAccount: TokenAccountPDA,
    userTokenAccount: token_ata,
    nftAccountPdaAddress: associatedTokenAccount,
  })
  // console.log((await builder.instruction()).keys);

  await buildAndSendTx({
    provider,
    ixs: [await builder.instruction()],
    extraSigners: [wallet],
  });
  console.log("userData: ",await program.account.userData.fetch(UserData));
});

it("decrease liquidity!", async () => {
  // console.log(await program.account.hybridPool.fetch(PoolAccountPDA))
    const pNftTransferClient = new PNftTransferClient(provider.connection, provider.wallet as anchor.Wallet,IDL,program.programId)

    const [tokenPDA,] = PublicKey.findProgramAddressSync(
      [Buffer.from("token-pool"),
      PoolAccountPDA.toBuffer(), 
      token_mint.toBuffer()],
      program.programId
    );
  
  // [b"token-pda-account", pool_account.key().as_ref(), hybrid_nft_account.key().as_ref()]
  const allowList2 = nft_mints
  
  let proof2=getMerkleProof(allowList2,pnft_mint_addr.toBuffer())
  const [nftPda,  nft_bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft-pool"),PoolAccountPDA.toBuffer(),pnft_mint_addr.toBuffer()], program.programId
  );
  // const nftPDAAccount = await getOrCreateAssociatedTokenAccount(
  //   provider.connection,
  //   wallet,
  //   token_mint,
  //   nftPda,
  //   true
  // );
  
  root=Buffer.from(getMerkleRoot([token_mint.toBuffer()]))
  const associatedTokenAccount = await getAssociatedTokenAddress(
    pnft_mint_addr,
    PoolAccountPDA,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
    const [nftPDA, tokenBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("token-pda-account"), PoolAccountPDA.toBuffer(), associatedTokenAccount.toBuffer()], program.programId)
      const PnftAssiciateTokenAccount = await getAssociatedTokenAddress(
        pnft_mint_addr,
        PoolAccountPDA,
        true
      );

      // const transaction = new Transaction().add(PnftAssiciateTokenAccount);

      // // Send the transaction
      // await sendAndConfirmTransaction(connection, transaction, [wallet]);
          
            const builder = await pNftTransferClient.removeLiquidity({
    nftMint: pnft_mint_addr,
    userAta:pnft_ata_addr,
    poolAta: nftPDA,
    owner: wallet.publicKey,
    pool:PoolAccountPDA,
    id:id_num,
    bump:PoolTokenAccountBump,
    proof:proof2,
    config:Config,
    userData: UserData,
    tokenMint: token_mint,
    hybridTokenAccount: tokenPDA,
    userTokenAccount: token_ata,
    nftAccountPdaAddress: associatedTokenAccount,
    // payar:wallet
  })
  //  console.log(builder);
  
  // console.log((await builder.instruction()).keys);
  
  await buildAndSendTx({
    provider,
    ixs: [await builder.instruction()],
    extraSigners: [wallet],
    // debug:true
  });

  console.log("userData: ",await program.account.userData.fetch(UserData));
});

const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor.getProvider().connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    commitment
  )
}

const confirmTxs = async (signatures: string[]) => {
  await Promise.all(signatures.map(confirmTx))
}


const newMintToAta = async (connection, minter: Keypair,decimals:number,amount:number,program_id:PublicKey): Promise<{ mint: PublicKey, ata: PublicKey }> => { 
  let keypair = new Keypair();
  
  const mint = await createMint(connection, minter, minter.publicKey, null, decimals,keypair,null,program_id)
  const ata = await createAccount(connection, minter, mint, minter.publicKey,null,null,program_id)
  const signature = await mintTo(connection, minter, mint, ata, minter, amount,[],null,program_id)
  
  await confirmTx(signature)
  
  return {
    mint,
    ata
  }
}
const mintNFTs = async (connection: Connection, minter: Keypair, count: number,program_id:PublicKey): Promise<{ mint: PublicKey, ata: PublicKey }[]> => { 
    let data: { mint: PublicKey, ata: PublicKey }[] = [];
    
    for (let index = 0; index < count; index++) {
        let nft = await newMintToAta(connection, minter,0,1,program_id);
        data.push({ mint: nft.mint, ata: nft.ata });
    }
    
    return data;
}

// // const newMintToAta = async (connection, minter: Keypair): Promise<{ mint: PublicKey, ata: PublicKey }> => { 
// //   const mint = await createMint(connection, minter, minter.publicKey, null, 6)
// //   // await getAccount(connection, mint, commitment)
// //   const ata = await createAccount(connection, minter, mint, minter.publicKey)
// //   const signature = await mintTo(connection, minter, mint, ata, minter, 21e12)
// //   await confirmTx(signature)
// //   return {
// //     mint,
// //     ata
// //   }
// // }
// // const tickCalculation =  (amount_based_b:any,tick_spacing:number)=> { 
// //     const base = 1.0001;
// //     const x = Math.log(amount_based_b) / Math.log(base);
// //     let tick=Math.round(x/tick_spacing) * tick_spacing
// //     return tick;

// // }
