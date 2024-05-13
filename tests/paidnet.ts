import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Paidnet } from "../target/types/paidnet";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("paidnet", () => {
  // Configure the client to use the local cluster.
  function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms))
  }
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();

  const connection = provider.connection;

  anchor.setProvider(provider);

  const program = anchor.workspace.Paidnet as Program<Paidnet>;

  const owner = provider.wallet as NodeWallet;

  const idoMint = new PublicKey("7iRnKFvRgzbmMbbgYYR9QSANLHvXaV9Lv7QnPe7fyjsE");
  const purchaseMint = new PublicKey("CBNdwTxCwVazfUQgzXZ8ZVbeg25prC5aHrdekFrmg6TD");
  const decimals = 9;

  const pool = Keypair.generate();

  it("Create Pool!", async () => {
    // Add your test here.
    const maxPurchaseForKycUser = new BN(100 * 10 ** decimals);
    const maxPurchaseForNotKycuser = new BN(10 * 10 ** decimals);
    const tokenFee = new BN(0);
    const galaxyFee = new BN(0);
    const crowdFee = new BN(0);
    const galaxyProportion = new BN(5000);
    const earlyProportion = new BN(5000);
    const totalRaiseAmount = new BN(100 * 10 ** decimals);
    const whaleOpen = new BN(Math.floor(Date.now() / 1000));
    const whaleClose = new BN(10);
    const communtiyClose = new BN(10);
    const rate = new BN(1);
    const currencyDecimals = new BN(9);

    const tgeDate = new BN(Math.floor(Date.now() / 1000) + 30);
    const tgePercentage = new BN(100);
    const vestingCliff = tgeDate.add(new BN(1));
    const vestingFrequency = new BN(0);
    const numberOfVesting = new BN(2);
    const [vestingStorageAccount, vesting_bump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("vesting-storage"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );

    const tx = await program.methods.createPool(
      [
        maxPurchaseForKycUser,
        maxPurchaseForNotKycuser,
        tokenFee,
        galaxyFee,
        crowdFee,
        galaxyProportion,
        earlyProportion,
        totalRaiseAmount,
        whaleOpen,
        whaleClose,
        communtiyClose,
        rate,
        currencyDecimals,
        tgeDate,
        tgePercentage,
        vestingCliff,
        vestingFrequency,
        numberOfVesting
      ]
    )
    .accounts({
      purchaseMint,
      idoMint,
      poolStorageAccount: pool.publicKey,
      vestingStorageAccount,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID
    }).signers([pool]).rpc();
    console.log(tx);
  });

  it ("Update Time", async () => {
    const whaleCloseTime = new BN(Math.floor(Date.now() / 1000) + 12);
    const communityCloseTime = new BN(Math.floor(Date.now() / 1000) + 12);
    const [vestingStorageAccount, vesting_bump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("vesting-storage"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );

    const tx = await program.methods.updateTime(
      whaleCloseTime,
      communityCloseTime
    ).accounts({
      poolStorageAccount: pool.publicKey,
      vestingStorageAccount
    }).rpc();
    console.log(tx);
  });

  it ("Update TGE date", async () => {
    const tgeDate = new BN(Math.floor(Date.now() / 1000) + 20);
    const [vestingStorageAccount, vesting_bump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("vesting-storage"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );
    const tx = await program.methods.updateTgeDate(
      tgeDate
    ).accounts({
      poolStorageAccount: pool.publicKey,
      vestingStorageAccount
    }).rpc();
    console.log(tx);
  });
  
  it ("Fund IDO Token", async () => {
    const amount = new BN(100 * 10 ** 9);
    const [vestingStorageAccount, vesting_bump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("vesting-storage"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );
    const [idoVault, bump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("ido-vault"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );

    const userToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      idoMint,
      owner.publicKey
    )
    const tx = await program.methods.fundIdoToken(
      amount,
      bump
    ).accounts({
      idoMint,
      userToken: userToken.address,
      vestingStorageAccount,
      idoVault,
      poolStorageAccount: pool.publicKey
    }).rpc();
    console.log("Your transaction signature", tx);
  });
  it ("Buy Token in early pool", async () => {
    const amount = new BN(100 * 10 ** 9);

    const userPurchaseToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      purchaseMint,
      owner.publicKey
    );

    const [vestingStorageAccount, vesting_bump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("vesting-storage"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );


    const [purchaseVault, purchaseBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("purchase-vault"),
        pool.publicKey.toBuffer()
      ],
      program.programId
    );

    const [userPurchaseAccount, _] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("user-purchase"),
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );

    const [userVesting, __] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("user-vesting"),
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );


    const tx = await program.methods.buyTokenInEarlyPool(
      amount,
      purchaseBump
    ).accounts({
      idoMint,
      userPurchaseToken: userPurchaseToken.address,
      vestingStorageAccount,
      poolStorageAccount: pool.publicKey,
      purchaseVault,
      userPurchaseAccount,
      purchaseMint,
      userVesting
    }).rpc().catch(e => console.log(e));
    console.log("Your transaction signature", tx);
    const vesting = await program.account.userVestingAccount.all();
    const vestingData = {
      publicKey: vesting[0].publicKey.toBase58(),
      totalAmount: vesting[0].account.totalAmount.toString(),
      claimedAmount: vesting[0].account.claimedAmount.toString()
    }
    console.table(vestingData);
  });
});
