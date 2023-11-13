import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CnftVault } from "../target/types/cnft_vault";
import { keypairIdentity, Metaplex } from "@metaplex-foundation/js";
import {
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID
} from "@solana/spl-account-compression";
import { AccountMeta, Keypair, PublicKey } from "@solana/web3.js";
import console from "console";
import { findTreeConfigPda, MPL_BUBBLEGUM_PROGRAM_ID } from "@metaplex-foundation/mpl-bubblegum";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

import { PublicKey as UmiPK } from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID, } from "@metaplex-foundation/mpl-token-metadata";
import { WrappedConnection } from "./WrappedConnection";

// todo import from somewhere
const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
// todo import from somewhere
const ASSOCIATED_TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);


function bufferToArray(buffer: Buffer): number[] {
  const nums: number[] = [];
  for (let i = 0; i < buffer.length; i++) {
    nums.push(buffer[i]);
  }
  return nums;
}

export function decode(stuff: string) {
  return bufferToArray(bs58.decode(stuff))
}

describe("nft_program", () => {
  // a few constants that need to be filled for the test to work
  const authorityWallet = Keypair.fromSecretKey('<tree and collection authority secret key here>')
  const collectionMint = new anchor.web3.PublicKey('<your collection mint>')
  const metadataAddress = new anchor.web3.PublicKey('<your collection metadata account>')
  const masterEditionAddress = new anchor.web3.PublicKey('<your collection edition account>')
  const leafOwner = Keypair.fromSecretKey('<recipient of the cNFT>');
  const merkleTree = new anchor.web3.PublicKey('<merkle tree account>');
  const treeConfig = new anchor.web3.PublicKey('<tree config account>');
  // this needs to be filled for the burn test to work, you can get the asset id of the cNFT you want to burn by running the mint test first and noting it down
  const assetId = '<address of the cNFT you want to burn>'

  // NFT metadata
  const name = "Road"
  const symbol = "RD"
  const uri = "https://arweave.net/Apu1g7uhv52CMeQNfevoody9dVDmaWtQ3TklI6cbNRM"
  const sellerFeeBasisPoints = 0

  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CnftVault as Program<CnftVault>;

  const metaplex = Metaplex.make(provider.connection).use(keypairIdentity(authorityWallet))

  const centralAuthority = PublicKey.findProgramAddressSync([Buffer.from("central_authority")], program.programId)[0];
  const [bubblegumSigner, _] = PublicKey.findProgramAddressSync(
    // `collection_cpi` is a custom prefix required by the Bubblegum program
    [Buffer.from("collection_cpi", "utf8")],
    new anchor.web3.PublicKey(MPL_BUBBLEGUM_PROGRAM_ID)
  );

  // const metadataAddress = anchor.web3.PublicKey.findProgramAddressSync(
  //   [
  //     Buffer.from("metadata"),
  //     TOKEN_METADATA_PROGRAM_ID.toBuffer(),
  //     collectionMint.toBuffer(),
  //   ],
  //   TOKEN_METADATA_PROGRAM_ID
  // )[0];
  // console.log("Metadata:", metadataAddress.toBase58());
  //
  // const masterEditionAddress = anchor.web3.PublicKey.findProgramAddressSync(
  //   [
  //     Buffer.from("metadata"),
  //     TOKEN_METADATA_PROGRAM_ID.toBuffer(),
  //     collectionMint.toBuffer(),
  //     Buffer.from("edition"),
  //   ],
  //   TOKEN_METADATA_PROGRAM_ID
  // )[0];
  // console.log("Master edition:", masterEditionAddress.toBase58());

  it("Should initialize a collection", async () => {
    return;
    // Derive the mint address and the associated token account address

    console.log(`Central authority: ${centralAuthority.toBase58()}`);
    const mintKeypair: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    const associatedTokenAccount = anchor.utils.token.associatedAddress({
      mint: mintKeypair.publicKey,
      owner: centralAuthority,
    });
    console.log(`Mint: ${mintKeypair.publicKey}`);

    // Derive the metadata and master edition addresses

    const metadataAddress = (await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    ))[0];
    console.log("Metadata: ", metadataAddress.toBase58());
    const masterEditionAddress = (await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    ))[0];
    console.log("Master edition:", masterEditionAddress.toBase58());

    const tx = await program.methods.initialize(
        "Collection",
        "COL",
        "https://arweave.net/Rmg4pcIv-0FQ7M7X838p2r592Q4NU63Fj7o7XsvBHEE",
      )
      .accounts(
        {
          signer: authorityWallet.publicKey,
          centralAuthority,
          mint: mintKeypair.publicKey,
          associatedTokenAccount,
          metadataAccount: metadataAddress,
          masterEditionAccount: masterEditionAddress,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
      .signers([authorityWallet, mintKeypair])
      .rpc()
    console.log(tx)
  })

  it("Mints a cnft to an existing tree and collection", async () => {
    return;
    // return;
    const umi = createUmi(provider.connection.rpcEndpoint);
    const treeConfig = findTreeConfigPda(
      umi,
      {
        merkleTree: merkleTree.toBase58() as UmiPK,
      }
    )[0]

    const tx = await program.methods
      .mintCnft(
        name,
        symbol,
        uri,
        sellerFeeBasisPoints,
      )
      .accounts({
        treeConfig,
        leafOwner: leafOwner.publicKey,
        merkleTree,
        treeDelegate: authorityWallet.publicKey,
        centralAuthority,
        collectionAuthorityRecordPda: MPL_BUBBLEGUM_PROGRAM_ID,
        collectionMint,
        collectionMetadata: metadataAddress,
        editionAccount: masterEditionAddress,
        bubblegumSigner,
        logWrapper: SPL_NOOP_PROGRAM_ID,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        bubblegumProgram: MPL_BUBBLEGUM_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authorityWallet])
      .rpc()
    console.log(tx)
  })

  it("Burns an existing cnft", async () => {
    // todo fix the burn call
    const connection = new WrappedConnection(provider.connection.rpcEndpoint)
    let assetProof = await connection.getAssetProof(assetId);
    const treeAccount = await ConcurrentMerkleTreeAccount.fromAccountAddress(
      connection,
      merkleTree,
    )
    const canopyDepth = treeAccount.getCanopyDepth()
    const proof: AccountMeta[] = assetProof.proof
      .slice(0, assetProof.proof.length - (!!canopyDepth ? canopyDepth : 0))
      .map((node: string) => ({
        pubkey: new PublicKey(node),
        isSigner: false,
        isWritable: false,
      }));

    const rpcAsset = await connection.getAsset(assetId);
    const umi = createUmi(provider.connection.rpcEndpoint);
    const treeConfig = findTreeConfigPda(
      umi,
      {
        merkleTree: merkleTree.toBase58() as UmiPK,
      }
    )[0]

    const treeConfigPublicKey = new anchor.web3.PublicKey(treeConfig)
    console.log('treeConfigPublicKey', treeConfigPublicKey.toBase58())
    const root = [...new PublicKey(assetProof.root.trim()).toBytes()]
    const dataHash = [...new PublicKey(rpcAsset.compression.data_hash.trim()).toBytes()]
    const creatorHash = [
      ...new PublicKey(rpcAsset.compression.creator_hash.trim()).toBytes(),
    ]

    const nonce = new anchor.BN(rpcAsset.compression.leaf_id);
    const index = rpcAsset.compression.leaf_id;

    const tx = await program.methods
      .burnCnft(
        root,
        dataHash,
        creatorHash,
        nonce,
        index,
      )
      .accounts({
        treeConfig: treeConfigPublicKey,
        leafOwner: leafOwner.publicKey,
        merkleTree,
        logWrapper: SPL_NOOP_PROGRAM_ID,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        bubblegumProgram: MPL_BUBBLEGUM_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([leafOwner]) // here we sign it with the leaf delegate, should be the owner
      .remainingAccounts(proof)
      .rpc({
        skipPreflight: true,
      })

    console.log(tx)
  })
});