import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ExtensionType,
  LENGTH_SIZE,
  TOKEN_2022_PROGRAM_ID,
  TYPE_SIZE,
  getAssociatedTokenAddressSync,
  getMintLen,
} from "@solana/spl-token";
import { TokenMetadata, pack } from "@solana/spl-token-metadata";
import {
  Keypair,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { StableSocialToken } from "../target/types/stable_social_token";

describe("stable-social-token", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .StableSocialToken as Program<StableSocialToken>;
  const wallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  const randomKey = Keypair.generate().publicKey;
  const [mint] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint"), randomKey.toBuffer()],
    program.programId
  );
  const [authority] = PublicKey.findProgramAddressSync(
    [Buffer.from("authority"), mint.toBuffer()],
    program.programId
  );
  const USDC = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
  const authorityStableTokenAccount = getAssociatedTokenAddressSync(
    USDC,
    authority,
    true
  );

  it("Create Mint!", async () => {
    const mintLen = getMintLen([
      ExtensionType.TransferFeeConfig,
      ExtensionType.MetadataPointer,
    ]);
    // Add your test here.
    const ix = await program.methods
      .createMint({
        randomKey: randomKey,
        size: mintLen,
        transferFeeArgs: {
          feeBasisPts: 5,
          maxFee: new anchor.BN(Number.MAX_SAFE_INTEGER),
        },
        transferHookArgs: null,
      })
      .accounts({
        payer: wallet.publicKey,
      })
      .instruction();
    const transaction = new Transaction().add(ix);
    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Create Mint Metadata!", async () => {
    const metaData: TokenMetadata = {
      updateAuthority: wallet.publicKey,
      mint: mint,
      name: "OPOS",
      symbol: "OPOS",
      uri: "https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json",
      additionalMetadata: [["description", "Only Possible On Solana"]],
    };
    const metadataExtension = TYPE_SIZE + LENGTH_SIZE;
    // Size of metadata
    const metadataLen = pack(metaData).length;
    const additional_lamport =
      await connection.getMinimumBalanceForRentExemption(
        metadataExtension + metadataLen
      );
    const ix = await program.methods
      .createMintMetadata(
        new anchor.BN(additional_lamport),
        metaData.name,
        metaData.symbol,
        metaData.uri
      )
      .accounts({
        mint: mint,
        payer: wallet.publicKey,
      })
      .instruction();

    const transaction = new Transaction().add(ix);
    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Issue Mint!", async () => {
    const payerStableTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey,
      false
    );
    const payerMintTokenAccount = getAssociatedTokenAddressSync(
      mint,
      wallet.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    const ix = await program.methods
      .issueMint(new anchor.BN(1 * 10 ** 6))
      .accounts({
        mint: mint,
        payer: wallet.publicKey,
        authorityStableCoinTokenAccount: authorityStableTokenAccount,
        payerMintTokenAccount: payerMintTokenAccount,
        payerStableCoinTokenAccount: payerStableTokenAccount,
      })
      .instruction();

    const transaction = new Transaction().add(ix);
    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Redeem Stablecoin!", async () => {
    const payerStableTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey
    );
    const payerMintTokenAccount = getAssociatedTokenAddressSync(
      mint,
      wallet.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    const feeCollectorStableCoinTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey
    );
    const ix = await program.methods
      .redeemStablecoin(new anchor.BN(1 * 10 ** 6))
      .accounts({
        mint: mint,
        payer: wallet.publicKey,
        authorityStableCoinTokenAccount: authorityStableTokenAccount,
        payerMintTokenAccount: payerMintTokenAccount,
        payerStableCoinTokenAccount: payerStableTokenAccount,
        feeCollector: wallet.publicKey,
        feeCollectorStableCoinTokenAccount: feeCollectorStableCoinTokenAccount,
      })
      .instruction();

    const transaction = new Transaction().add(ix);
    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );
    console.log(`Transaction Signature: ${txSig}`);
  });
});
