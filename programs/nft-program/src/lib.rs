use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::pubkey::Pubkey,
    system_program,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata,
        mpl_token_metadata::types::{CollectionDetails, DataV2},
    },
    token::{Mint, mint_to, MintTo, Token, TokenAccount},
};
use mpl_bubblegum::instructions::{BurnCpiBuilder, CreateTreeConfigCpiBuilder, MintToCollectionV1CpiBuilder};
use mpl_bubblegum::types::{Collection, MetadataArgs, TokenProgramVersion, TokenStandard};
use mpl_token_metadata;
use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
};

declare_id!("HcmjtyqZgSeNFdKvHCBCDNEJHSwrf9KveBrbXQKXPxqN");

// The program will support only trees of the following parameters:
const MAX_TREE_DEPTH: u32 = 14;
const MAX_TREE_BUFFER_SIZE: u32 = 64;
// this corresponds to account with a canopy depth 11.
// If you need the tree parameters to be dynamic, you can use the following function:
// fn tree_bytes_size() -> usize {
//     const CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1: usize = 2 + 54;
//     let merkle_tree_size = size_of::<ConcurrentMerkleTree<14, 64>>();
//     msg!("merkle tree size: {}", merkle_tree_size);
//     let canopy_size = ((2 << 9) - 2) * 32;
//     msg!("canopy size: {}", canopy_size);
//     CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1 + merkle_tree_size + (canopy_size as usize)
// }
const REQUIRED_TREE_ACCOUNT_SIZE: usize = 162_808;


#[program]
pub mod cnft_vault {
    use super::*;

    // initializes the basic contract data and creates a maintained NFT collection
    pub fn initialize(
        ctx: Context<Init>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let bump_seed = [ctx.bumps.central_authority];
        let signer_seeds: &[&[&[u8]]] = &[&[
            "central_authority".as_bytes(),
            &bump_seed.as_ref(),
        ]];
        // create mint account
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.central_authority.to_account_info(),
            },
            signer_seeds,
        );

        mint_to(cpi_context, 1)?;

        // create metadata account
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.central_authority.to_account_info(),
                update_authority: ctx.accounts.central_authority.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        );

        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(
            cpi_context,
            data_v2,
            true,
            true,
            Some(CollectionDetails::V1 { size: 1 }),
        )?;

        //create master edition account
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.central_authority.to_account_info(),
                mint_authority: ctx.accounts.central_authority.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        );

        create_master_edition_v3(cpi_context, Some(0))?;

        ctx.accounts.central_authority.collection_address = ctx.accounts.mint.key();
        Ok(())
    }

    pub fn initialize_tree<'info>(ctx: Context<'_, '_, '_, 'info, MerkleTree<'info>>) -> Result<()> {
        msg!("initializing merkle tree");
        require_eq!(ctx.accounts.merkle_tree.data.borrow().len(), REQUIRED_TREE_ACCOUNT_SIZE, MyError::UnsupportedTreeAccountSize);
        let bump_seed = [ctx.bumps.central_authority];
        let signer_seeds: &[&[&[u8]]] = &[&[
            "central_authority".as_bytes(),
            &bump_seed.as_ref(),
        ]];

        CreateTreeConfigCpiBuilder::new(
            &ctx.accounts.bubblegum_program.to_account_info(),
        )
            .tree_config(&ctx.accounts.tree_config.to_account_info())
            .merkle_tree(&ctx.accounts.merkle_tree.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .tree_creator(&ctx.accounts.central_authority.to_account_info())
            .log_wrapper(&ctx.accounts.log_wrapper.to_account_info())
            .compression_program(&ctx.accounts.compression_program.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .max_depth(MAX_TREE_DEPTH)
            .max_buffer_size(MAX_TREE_BUFFER_SIZE)
            .invoke_signed(signer_seeds)?;

        ctx.accounts.central_authority.merkle_tree_address = Some(ctx.accounts.merkle_tree.key());
        Ok(())
    }

    // this instruction is permissionless. In a real-world program you would want to check the caller.
    pub fn mint_cnft<'info>(ctx: Context<'_, '_, '_, 'info, MintCNft<'info>>,
                            name: String,
                            symbol: String,
                            uri: String,
                            seller_fee_basis_points: u16) -> Result<()> {
        msg!("minting nft");
        require!(ctx.accounts.central_authority.merkle_tree_address.is_some(), MyError::InvalidMerkleTree);
        require_keys_eq!(*ctx.accounts.merkle_tree.key, ctx.accounts.central_authority.merkle_tree_address.unwrap(), MyError::InvalidMerkleTree);
        require_keys_eq!(*ctx.accounts.collection_mint.key, ctx.accounts.central_authority.collection_address, MyError::InvalidMerkleTree);
        let bump_seed = [ctx.bumps.central_authority];
        let signer_seeds: &[&[&[u8]]] = &[&[
            "central_authority".as_bytes(),
            &bump_seed.as_ref(),
        ]];
        MintToCollectionV1CpiBuilder::new(
            &ctx.accounts.bubblegum_program.to_account_info(),
        )
            .tree_config(&ctx.accounts.tree_config.to_account_info())
            .leaf_owner(&ctx.accounts.leaf_owner.to_account_info())
            .leaf_delegate(&ctx.accounts.leaf_owner.to_account_info())
            .merkle_tree(&ctx.accounts.merkle_tree.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .tree_creator_or_delegate(&ctx.accounts.central_authority.to_account_info())
            .collection_authority(&ctx.accounts.central_authority.to_account_info())
            .collection_authority_record_pda(Some(&ctx.accounts.bubblegum_program.to_account_info()))
            .collection_mint(&ctx.accounts.collection_mint.to_account_info())
            .collection_metadata(&ctx.accounts.collection_metadata.to_account_info())
            .collection_edition(&ctx.accounts.edition_account.to_account_info())
            .bubblegum_signer(&ctx.accounts.bubblegum_signer.to_account_info())
            .log_wrapper(&ctx.accounts.log_wrapper.to_account_info())
            .compression_program(&ctx.accounts.compression_program.to_account_info())
            .token_metadata_program(&ctx.accounts.token_metadata_program.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .metadata(
                MetadataArgs {
                    name,
                    symbol,
                    uri,
                    creators: vec![],
                    seller_fee_basis_points,
                    primary_sale_happened: false,
                    is_mutable: false,
                    edition_nonce: Some(0),
                    uses: None,
                    collection: Some(Collection {
                        verified: true,
                        key: ctx.accounts.collection_mint.key(),
                    }),
                    token_program_version: TokenProgramVersion::Original,
                    token_standard: Some(TokenStandard::NonFungible),
                }
            )
            .invoke_signed(signer_seeds)?;
        Ok(())
    }

    pub fn burn_cnft<'info>(ctx: Context<'_, '_, '_, 'info, BurnAccs<'info>>,
                            root: [u8; 32],
                            data_hash: [u8; 32],
                            creator_hash: [u8; 32],
                            nonce: u64,
                            index: u32) -> Result<()> {
        msg!("burning nft");
        require!(ctx.accounts.central_authority.merkle_tree_address.is_some(), MyError::InvalidMerkleTree);
        require_keys_eq!(*ctx.accounts.merkle_tree.key, ctx.accounts.central_authority.merkle_tree_address.unwrap(), MyError::InvalidMerkleTree);

        let remaining_accounts: Vec<(&AccountInfo, bool, bool)> = ctx.remaining_accounts
            .iter()
            .map(|account| (account, account.is_signer, account.is_writable))
            .collect();

        BurnCpiBuilder::new(
            &ctx.accounts.bubblegum_program.to_account_info(),
        )
            .tree_config(&ctx.accounts.tree_config.to_account_info())
            .leaf_owner(&ctx.accounts.leaf_owner.to_account_info(), true)
            .leaf_delegate(&ctx.accounts.leaf_owner.to_account_info(), true)
            .merkle_tree(&ctx.accounts.merkle_tree.to_account_info())
            .log_wrapper(&ctx.accounts.log_wrapper.to_account_info())
            .compression_program(&ctx.accounts.compression_program.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .add_remaining_accounts(&remaining_accounts)
            .root(root)
            .data_hash(data_hash)
            .creator_hash(creator_hash)
            .nonce(nonce)
            .index(index)
            .invoke()?;

        Ok(())
    }
}

#[error_code]
pub enum MyError {
    #[msg("No signer")]
    NoSigner,
    #[msg("Unsupported tree account size")]
    UnsupportedTreeAccountSize,
    #[msg("Invalid merkle tree")]
    InvalidMerkleTree,
    #[msg("Invalid collection")]
    InvalidCollection,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MintParams {
    uri: String,
}

#[derive(Clone)]
pub struct MplBubblegum;

impl Id for MplBubblegum {
    fn id() -> Pubkey {
        mpl_bubblegum::ID
    }
}

#[derive(Clone)]
pub struct MplTokenMetadata;

impl Id for MplTokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}

#[derive(Clone)]
pub struct Noop;

impl Id for Noop {
    fn id() -> Pubkey {
        Pubkey::from_str("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV").unwrap()
    }
}

#[derive(Clone)]
pub struct SplAccountCompression;

impl Id for SplAccountCompression {
    fn id() -> Pubkey {
        Pubkey::from_str("cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK").unwrap()
    }
}

#[derive(Accounts)]
pub struct MintCNft<'info> {
    pub payer: Signer<'info>,

    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,

    /// CHECK: This account is neither written to nor read from.
    pub leaf_owner: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: unsafe
    pub merkle_tree: UncheckedAccount<'info>,

    pub tree_delegate: Signer<'info>,

    #[account(
    seeds = [b"central_authority"],
    bump
    )]
    pub central_authority: Account<'info, CentralStateData>,

    /// CHECK: This account is checked in the instruction
    pub collection_mint: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: This account is checked in the instruction
    pub edition_account: UncheckedAccount<'info>,

    /// CHECK: This is just used as a signing PDA.
    pub bubblegum_signer: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub token_metadata_program: Program<'info, MplTokenMetadata>,
    pub bubblegum_program: Program<'info, MplBubblegum>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnAccs<'info> {
    #[account(
    seeds = [b"central_authority"],
    bump
    )]
    pub central_authority: Account<'info, CentralStateData>,
    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub leaf_owner: Signer<'info>,
    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    /// CHECK: This account is modified in the downstream program
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the instruction
    pub tree_config: UncheckedAccount<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub bubblegum_program: Program<'info, MplBubblegum>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct CentralStateData {
    pub collection_address: Pubkey,
    pub merkle_tree_address: Option<Pubkey>,
}

impl CentralStateData {
    pub const MAX_SIZE: usize = 32 * 3;
}

#[derive(Accounts)]
pub struct Init<'info> {
    /// CHECK: ok, we are passing in this account ourselves
    #[account(mut, signer)]
    pub signer: AccountInfo<'info>,
    #[account(
    init,
    payer = signer,
    space = 8 + CentralStateData::MAX_SIZE,
    seeds = [b"central_authority"],
    bump
    )]
    pub central_authority: Account<'info, CentralStateData>,
    #[account(
    init,
    payer = signer,
    mint::decimals = 0,
    mint::authority = central_authority.key(),
    mint::freeze_authority = central_authority.key(),
    )]
    pub mint: Account<'info, Mint>,
    #[account(
    init_if_needed,
    payer = signer,
    associated_token::mint = mint,
    associated_token::authority = central_authority
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    /// CHECK - address
    #[account(
    mut,
    address = find_metadata_account(& mint.key()).0,
    )]
    pub metadata_account: AccountInfo<'info>,
    /// CHECK: address
    #[account(
    mut,
    address = find_master_edition_account(& mint.key()).0,
    )]
    pub master_edition_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MerkleTree<'info> {
    #[account(mut, signer)]
    pub payer: Signer<'info>,

    #[account(
    seeds = [b"central_authority"],
    bump,
    mut
    )]
    pub central_authority: Account<'info, CentralStateData>,

    /// CHECK: This account must be all zeros
    #[account(
    zero,
    signer
    )]
    pub merkle_tree: AccountInfo<'info>,

    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,

    // program
    pub bubblegum_program: Program<'info, MplBubblegum>,
    pub system_program: Program<'info, System>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
}