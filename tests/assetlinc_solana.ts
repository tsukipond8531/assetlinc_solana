import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Subscription } from "../target/types/subscription";
import { assert } from "chai";

describe("subscription", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const user = provider.wallet as anchor.Wallet;
    const program = anchor.workspace.Subscription as Program<Subscription>;
    const feeReceiver = anchor.web3.Keypair.generate();
    const userDataPublicKey = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user-data"), user.publicKey.toBuffer()],
        program.programId
    )[0];

    it("Can not create new subscriptions with invalid tier", async () => {
        const invalidTier = 3;

        try {
            await program.methods
                .manageSubscription(invalidTier)
                .accounts({
                    user: user.publicKey,
                    feeReceiver: feeReceiver.publicKey,
                })
                .signers([user.payer])
                .rpc();
        } catch (err) {
            assert(
                err.message ===
                    "AnchorError occurred. Error Code: InvalidSubscriptionTier. Error Number: 6000. Error Message: Invalid subscription tier.."
            );
        }
    });

    it("Can create new subscriptions with tier 0", async () => {
        const tier = 0;

        await program.methods
            .manageSubscription(tier)
            .accounts({
                user: user.publicKey,
                feeReceiver: feeReceiver.publicKey,
            })
            .signers([user.payer])
            .rpc();

        const userData = await program.account.userData.fetch(userDataPublicKey);
        assert(userData.subscriptionTier === tier);
        assert(Number(userData.lastPayment) !== 0);
    });

    it("Can create new subscriptions with tier 1", async () => {
        const tier = 1;

        await program.methods
            .manageSubscription(tier)
            .accounts({
                user: user.publicKey,
                feeReceiver: feeReceiver.publicKey,
            })
            .signers([user.payer])
            .rpc();

        const userData = await program.account.userData.fetch(userDataPublicKey);
        assert(userData.subscriptionTier === tier);
        assert(Number(userData.lastPayment) !== 0);
    });

    it("Can create new subscriptions with tier 0", async () => {
        const tier = 2;

        await program.methods
            .manageSubscription(tier)
            .accounts({
                user: user.publicKey,
                feeReceiver: feeReceiver.publicKey,
            })
            .signers([user.payer])
            .rpc();

        const userData = await program.account.userData.fetch(userDataPublicKey);
        assert(userData.subscriptionTier === tier);
        assert(Number(userData.lastPayment) !== 0);
    });

    it("Can register asset as Nft", async () => {
        const assetName = "My nft";
        const assetValue = new anchor.BN(1e9); // 1 SOL
        const royalty = Math.floor((assetValue.toNumber() * 3) / 100);
        const feeReceiverBalanceBefore = await provider.connection.getBalance(
            feeReceiver.publicKey
        );

        await program.methods
            .registerAssetAsNft(assetName, assetValue)
            .accounts({
                user: user.publicKey,
                feeReceiver: feeReceiver.publicKey,
            })
            .rpc();
        const nftDataPublicKey = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("nft-data"), user.publicKey.toBuffer(), Buffer.from(assetName)],
            program.programId
        )[0];
        const nftData = await program.account.nftData.fetch(nftDataPublicKey);
        const feeReceiverBalanceAfter = await provider.connection.getBalance(feeReceiver.publicKey);
        assert(nftData.name === assetName);
        assert(Number(nftData.value) === assetValue.toNumber());
        assert(nftData.owner.toString() === user.publicKey.toString());
        assert(feeReceiverBalanceAfter - feeReceiverBalanceBefore === royalty);
    });

    it("Can check subscription", async () => {
        const tier = 0;

        await program.methods
            .manageSubscription(tier)
            .accounts({
                user: user.publicKey,
                feeReceiver: feeReceiver.publicKey,
            })
            .signers([user.payer])
            .rpc();

        await program.methods
            .checkSubscription()
            .accounts({
                userData: userDataPublicKey,
            })
            .rpc();
    });
});
