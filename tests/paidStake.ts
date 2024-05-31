import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PaidStake } from '../target/types/paid_stake';
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("paid stake", () => {
  // Configure the client to use the local cluster.
  function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms))
  }
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();

  const connection = provider.connection;

  anchor.setProvider(provider);

  const program = anchor.workspace.PaidStake as Program<PaidStake>;

  const owner = provider.wallet as NodeWallet;

  const rewardMint = new PublicKey("7iRnKFvRgzbmMbbgYYR9QSANLHvXaV9Lv7QnPe7fyjsE");
  const stakeMint = new PublicKey("CBNdwTxCwVazfUQgzXZ8ZVbeg25prC5aHrdekFrmg6TD");
  const decimals = 9;

  const  pool = Keypair.generate();

  it("Init Stake!", async () => {
    // Add your test here.
    const tx = await program.methods.initPool(
      9,
      9,
      new BN(100)
    ).accounts({
      rewardMint,
      stakeMint,
      pool: pool.publicKey
    }).signers([pool]).rpc();
    console.log("Your transaction signature", tx);
  });

  it ("Init reward", async () => {
    const [rewardPot, potBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("reward-pot"),
        pool.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ownerToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      rewardMint,
      owner.publicKey
    )

    const tx = await program.methods.initReward(
      new BN(100 * 10 ** 9),
      potBump
    ).accounts({
      rewardMint,
      ownerToken: ownerToken.address,
      pool: pool.publicKey,
      rewardPot,
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it ("Add reward", async () => {
    const [rewardPot, potBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("reward-pot"),
        pool.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ownerToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      rewardMint,
      owner.publicKey
    )

    const tx = await program.methods.addReward(
      new BN(100 * 10 ** 9)
    ).accounts({
      rewardMint,
      ownerToken: ownerToken.address,
      pool: pool.publicKey,
      rewardPot
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it ("init staker", async () => {
    const [staker, _] = PublicKey.findProgramAddressSync(
      [
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );

    const tx = await program.methods.initStaker().accounts({
      pool: pool.publicKey,
      staker
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it ("init vault", async () => {
    const [stakeVault, vaultBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("farm-vault"),
        pool.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [staker, _] = PublicKey.findProgramAddressSync(
      [
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );

    const userToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      stakeMint,
      owner.publicKey
    )

    const tx = await program.methods.initVault(
      new BN(10 * 10 ** 9),
      vaultBump
    ).accounts({
      pool: pool.publicKey,
      stakeVault,
      stakeMint,
      staker,
      userToken: userToken.address
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it ("stake", async () => {
    const [stakeVault, vaultBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("farm-vault"),
        pool.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [staker, _] = PublicKey.findProgramAddressSync(
      [
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );

    const userToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      stakeMint,
      owner.publicKey
    )

    const tx = await program.methods.stake(
      new BN(10 * 10 ** 9),
    ).accounts({
      pool: pool.publicKey,
      stakeVault,
      stakeMint,
      staker,
      userToken: userToken.address
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it("claim", async () => {
    await sleep(3000);
    const userRewardToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      rewardMint,
      owner.publicKey
    );
    const [staker, _] = PublicKey.findProgramAddressSync(
      [
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );

    const [rewardPot, potBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("reward-pot"),
        pool.publicKey.toBuffer(),
      ],
      program.programId
    );


    const tx = await program.methods.claim().accounts({
      rewardMint,
      userRewardToken: userRewardToken.address,
      pool: pool.publicKey,
      staker,
      rewardPot
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it ("withdraw", async () => {
    const [staker, _] = PublicKey.findProgramAddressSync(
      [
        pool.publicKey.toBuffer(),
        owner.publicKey.toBuffer()
      ],
      program.programId
    );

    const userToken = await getOrCreateAssociatedTokenAccount(
      connection,
      owner.payer,
      stakeMint,
      owner.publicKey
    );

    const [stakeVault, vaultBump] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("farm-vault"),
        pool.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.methods.withdraw(
      new BN(20 * 10 ** 9)
    ).accounts({
      pool: pool.publicKey,
      stakeMint,
      staker,
      userStakeToken: userToken.address,
      stakeVault
    }).rpc();
  });

});
