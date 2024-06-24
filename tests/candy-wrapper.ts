import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ExtensionType,
  LENGTH_SIZE,
  TOKEN_2022_PROGRAM_ID,
  TYPE_SIZE,
  burnChecked,
  getAssociatedTokenAddressSync,
  getMintLen,
  getOrCreateAssociatedTokenAccount,
  harvestWithheldTokensToMint,
  transferChecked,
} from "@solana/spl-token";
import { TokenMetadata, pack } from "@solana/spl-token-metadata";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { CandyWrapper } from "../target/types/candy_wrapper";

describe("candy-wrapper", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.CandyWrapper as Program<CandyWrapper>;
  const wallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  const recipient = new PublicKey(
    "4gfBPGmnvGCpgnStMfwqxBbbdmKncGLy6DKN18qZVuH4"
  );
  const mintKeypair = Keypair.generate();
  // Address for Mint Account
  const mint = mintKeypair.publicKey;
  const [authority] = PublicKey.findProgramAddressSync(
    [Buffer.from("authority"), mint.toBuffer()],
    program.programId
  );
  const USDC = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
  const authorityBaseTokenAccount = getAssociatedTokenAddressSync(
    USDC,
    authority,
    true
  );
  const authorityMintTokenAccount = getAssociatedTokenAddressSync(
    mint,
    authority,
    true,
    TOKEN_2022_PROGRAM_ID
  );

  it("Set Protocol Fee", async () => {
    const txSig = await program.methods
      .setProtocolFee(500)
      .accounts({ payer: wallet.publicKey })
      .rpc({ skipPreflight: true });

    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Create Mint!", async () => {
    // these two are compulsory other extensions are optional
    const mintLen = getMintLen([
      ExtensionType.TransferFeeConfig,
      ExtensionType.MetadataPointer,
    ]);
    const lamports = await connection.getMinimumBalanceForRentExemption(
      mintLen
    );
    const ix1 = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: mint,
      space: mintLen,
      lamports,
      programId: TOKEN_2022_PROGRAM_ID,
    });
    const ix2 = await program.methods
      .createMint({
        admin: wallet.publicKey,
        mintToBaseRatio: 69,
        baseCoin: USDC,
        feeCollector: wallet.publicKey,
        issuanceFeeBasisPts: 0,
        redemptionFeeBasisPts: 500,
        transferFeeArgs: {
          feeBasisPts: 5,
          maxFee: new anchor.BN(Number.MAX_SAFE_INTEGER),
        },
      })
      .accounts({
        mint: mint,
        baseCoin: USDC,
        payer: wallet.publicKey,
      })
      .instruction();
    const transaction = new Transaction().add(ix1).add(ix2);
    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer, mintKeypair],
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
    const payerBaseTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey,
      false
    );
    const payerMintTokenAccount = getAssociatedTokenAddressSync(
      mint,
      wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    const feeCollectorBaseCoinTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey,
      true
    );
    const ix = await program.methods
      .issueMint(new anchor.BN(1 * 10 ** 6))
      .accounts({
        mint: mint,
        payer: wallet.publicKey,
        baseCoin: USDC,
        authorityBaseCoinTokenAccount: authorityBaseTokenAccount,
        payerMintTokenAccount: payerMintTokenAccount,
        payerBaseCoinTokenAccount: payerBaseTokenAccount,
        feeCollectorBaseCoinTokenAccount: feeCollectorBaseCoinTokenAccount,
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

  it("Transfer Mint!", async () => {
    const source = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const destination = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mint,
      recipient,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    await burnChecked(
      connection,
      wallet.payer,
      source.address,
      mint,
      wallet.publicKey,
      0.1 * 10 ** 6,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    const txSig = await transferChecked(
      connection,
      wallet.payer,
      source.address,
      mint,
      destination.address,
      wallet.publicKey,
      0.1 * 10 ** 6,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Redeem Basecoin!", async () => {
    const payerBaseTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey
    );
    const payerMintTokenAccount = getAssociatedTokenAddressSync(
      mint,
      wallet.publicKey,
      true,
      TOKEN_2022_PROGRAM_ID
    );
    const feeCollectorBaseCoinTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey
    );
    const ix = await program.methods
      .redeemBasecoin(new anchor.BN((1 * (9995 / 10000) - 0.1) * 10 ** 6))
      .accounts({
        mint: mint,
        payer: wallet.publicKey,
        baseCoin: USDC,
        authorityBaseCoinTokenAccount: authorityBaseTokenAccount,
        payerMintTokenAccount: payerMintTokenAccount,
        payerBaseCoinTokenAccount: payerBaseTokenAccount,
        feeCollectorBaseCoinTokenAccount: feeCollectorBaseCoinTokenAccount,
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

  it("Harvest fee to mint", async () => {
    const destination = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mint,
      recipient,
      false,
      "confirmed",
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    const txSig = await harvestWithheldTokensToMint(
      connection,
      wallet.payer, // Transaction fee payer
      mint, // Mint Account address
      [destination.address], // Source Token Accounts for fee harvesting
      undefined, // Confirmation options
      TOKEN_2022_PROGRAM_ID // Token Extension Program ID
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Withdraw to fee collector", async () => {
    const protocolBaseCoinTokenAccount =
      await getOrCreateAssociatedTokenAccount(
        connection,
        wallet.payer,
        USDC,
        wallet.publicKey,
        false
      );

    const feeCollectorBaseCoinTokenAccount = getAssociatedTokenAddressSync(
      USDC,
      wallet.publicKey
    );

    const ix = await program.methods
      .withdrawFees()
      .accounts({
        payer: wallet.publicKey,
        mint: mint,
        baseCoin: USDC,
        feeCollectorBaseCoinTokenAccount: feeCollectorBaseCoinTokenAccount,
        protocolBaseCoinTokenAccount: protocolBaseCoinTokenAccount.address,
        authorityMintTokenAccount: authorityMintTokenAccount,
        authorityBaseCoinTokenAccount: authorityBaseTokenAccount,
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

    console.log(await program.account.authority.fetch(authority));
  });
  it("Change Issuance Fee", async () => {
    const txSig = await program.methods
      .changeIssuanceFee(1)
      .accounts({ authority: authority, payer: wallet.publicKey })
      .rpc();

    console.log(`Transaction Signature: ${txSig}`);

    console.log(
      (await program.account.authority.fetch(authority)).issuanceFeeBasisPts
    );
  });

  it("Change Redemption Fee", async () => {
    const txSig = await program.methods
      .changeRedemptionFee(1)
      .accounts({ authority: authority, payer: wallet.publicKey })
      .rpc();

    console.log(`Transaction Signature: ${txSig}`);

    console.log(
      (await program.account.authority.fetch(authority)).redemptionFeeBasisPts
    );
  });
});
