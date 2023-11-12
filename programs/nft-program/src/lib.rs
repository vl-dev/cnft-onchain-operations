use anchor_lang::prelude::*;
use mpl_bubblegum::instructions::BurnCpiBuilder;
use mpl_bubblegum::instructions::MintToCollectionV1CpiBuilder;
use mpl_bubblegum::types::{Collection, Creator, MetadataArgs, TokenProgramVersion, TokenStandard};
use mpl_token_metadata;
use solana_program::pubkey::Pubkey;
use spl_account_compression::{
    Noop, program::SplAccountCompression,
};

declare_id!("HcmjtyqZgSeNFdKvHCBCDNEJHSwrf9KveBrbXQKXPxqN");

#[program]
pub mod cnft_vault {
    use super::*;

    pub fn mint_cnft<'info>(ctx: Context<'_, '_, '_, 'info, Mint<'info>>,
                            name: String,
                            symbol: String,
                            uri: String,
                            seller_fee_basis_points: u16) -> Result<()> {
        msg!("minting nft");
        let burn_ix = MintToCollectionV1CpiBuilder::new(
            &ctx.accounts.bubblegum_program.to_account_info(),
        )
            .tree_config(&ctx.accounts.tree_config.to_account_info())
            .leaf_owner(&ctx.accounts.leaf_owner.to_account_info())
            .leaf_delegate(&ctx.accounts.leaf_delegate.to_account_info())
            .merkle_tree(&ctx.accounts.merkle_tree.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .tree_creator_or_delegate(&ctx.accounts.tree_delegate.to_account_info())
            .collection_authority(&ctx.accounts.collection_authority.to_account_info())
            .collection_authority_record_pda(Some(&ctx
                .accounts
                .collection_authority_record_pda
                .to_account_info()))
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
                    creators: vec![Creator {
                        address: ctx.accounts.collection_authority.key(),
                        verified: true,
                        share: 100,
                    }],
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
            .invoke();
        Ok(())
    }


    pub fn burn_cnft<'info>(ctx: Context<'_, '_, '_, 'info, BurnAccs<'info>>,
                            root: [u8; 32],
                            data_hash: [u8; 32],
                            creator_hash: [u8; 32],
                            nonce: u64,
                            index: u32) -> Result<()> {
        msg!("burning nft");

        let remaining_accounts: Vec<(&AccountInfo, bool, bool)> = ctx.remaining_accounts
            .iter()
            .map(|account| (account, account.is_signer, account.is_writable))
            .collect();


        let burn_ix = BurnCpiBuilder::new(
            &ctx.accounts.bubblegum_program.to_account_info(),
        )
            .tree_config(&ctx.accounts.tree_config.to_account_info())
            .leaf_owner(&ctx.accounts.leaf_owner.to_account_info(), true)
            .leaf_delegate(&ctx.accounts.leaf_delegate.to_account_info(), false)
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
            .invoke();

        Ok(())
    }
}

#[error_code]
pub enum MyError {
    #[msg("No signer")]
    NoSigner
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

#[derive(Accounts)]
pub struct Mint<'info> {
    pub payer: Signer<'info>,

    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,

    /// CHECK: This account is neither written to nor read from.
    pub leaf_owner: AccountInfo<'info>,

    /// CHECK: This account is neither written to nor read from.
    pub leaf_delegate: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: unsafe
    pub merkle_tree: UncheckedAccount<'info>,

    pub tree_delegate: Signer<'info>,

    pub collection_authority: Signer<'info>,

    /// CHECK: Optional collection authority record PDA.
    /// If there is no collecton authority record PDA then
    /// this must be the Bubblegum program address.
    pub collection_authority_record_pda: UncheckedAccount<'info>,

    /// CHECK: This account is checked in the instruction
    pub collection_mint: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,
    //
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
    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub leaf_owner: Signer<'info>,
    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub leaf_delegate: Signer<'info>,
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