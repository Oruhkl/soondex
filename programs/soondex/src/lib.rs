use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::associated_token;
use crate::associated_token::AssociatedToken;
use integer_sqrt::IntegerSquareRoot;
use anchor_spl::token_interface::{transfer_checked, TransferChecked};




// Pool constants
pub const POOL_SEED: &[u8] = b"pool";
pub const MAX_FEE_RATE: u64 = 5000; // 50% in basis points
pub const TOTAL_FEE_RATE: u64 = 25; // 0.25% in basis points
pub const PROTOCOL_FEE_LAMPORTS: u64 = 150_000_000; // 0.15 SOL in lamports

declare_id!("FKczhwC9sbSnSKwG8Anp2NPsGCumTwbhABursN5a1dmX");

#[program]
pub mod soondex {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        token_x_mint: Pubkey,
        token_y_mint: Pubkey,
        fee_rate: u64,
    ) -> Result<()> {
        // CHECKS
        require!(fee_rate <= MAX_FEE_RATE, ErrorCode::InvalidFeeRate);
        require!(
            ctx.accounts.token_x_mint.key() == token_x_mint,
            ErrorCode::InvalidToken
        );
        require!(
            ctx.accounts.token_y_mint.key() == token_y_mint,
            ErrorCode::InvalidToken
        );
        require!(
            ctx.accounts.payer.lamports() >= PROTOCOL_FEE_LAMPORTS,
            ErrorCode::InsufficientFunds
        );
        
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        liquidity_pool.bump = ctx.bumps.liquidity_pool;
        
        // Set initial admin state
        liquidity_pool.authority = ctx.accounts.payer.key();
        liquidity_pool.super_admin = ctx.accounts.payer.key();
        liquidity_pool.admins = vec![ctx.accounts.payer.key()];

        // EFFECTS
        liquidity_pool.authority = ctx.accounts.payer.key();
        liquidity_pool.fee_rate = fee_rate;
        liquidity_pool.token_x_reserve = 0;
        liquidity_pool.token_y_reserve = 0;
        liquidity_pool.lp_token_supply = 0;
        liquidity_pool.token_x_mint = token_x_mint;
        liquidity_pool.token_y_mint = token_y_mint;

        // INTERACTIONS
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.protocol_wallet.to_account_info(),
                },
            ),
            PROTOCOL_FEE_LAMPORTS,
        )?;

        // Create associated token accounts
        let token_x_key = token_x_mint.key();
        let token_y_key = token_y_mint.key();
        
        let bump = Pubkey::find_program_address(
            &[
                POOL_SEED,
                token_x_key.as_ref(),
                token_y_key.as_ref()
            ],
            ctx.program_id
        ).1;
        
        let seeds = &[
            POOL_SEED,
            token_x_key.as_ref(),
            token_y_key.as_ref(),
            &[bump]
        ];

        // Create token X account
        let pool_token_x_address = associated_token::get_associated_token_address(
            &liquidity_pool.key(),
            &ctx.accounts.token_x_mint.key(),
        );

        if !ctx.accounts.pool_token_x_account.to_account_info().key.eq(&pool_token_x_address) {
            associated_token::create(CpiContext::new_with_signer(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    authority: liquidity_pool.to_account_info(),
                    associated_token: ctx.accounts.pool_token_x_account.to_account_info(),
                    mint: ctx.accounts.token_x_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[seeds],
            ))?;
        }

        // Create token Y account 
        let pool_token_y_address = associated_token::get_associated_token_address(
            &liquidity_pool.key(),
            &ctx.accounts.token_y_mint.key(),
        );

        if !ctx.accounts.pool_token_y_account.to_account_info().key.eq(&pool_token_y_address) {
            associated_token::create(CpiContext::new_with_signer(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    authority: liquidity_pool.to_account_info(),
                    associated_token: ctx.accounts.pool_token_y_account.to_account_info(),
                    mint: ctx.accounts.token_y_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[seeds],
            ))?;
        }

        emit!(PoolInitialized {
            authority: liquidity_pool.authority,
            fee_rate,
        });

        Ok(())
    }

    pub fn remove_pool(
        ctx: Context<RemovePool>,
        _token_x_mint: Pubkey,
        _token_y_mint: Pubkey,
    ) -> Result<()> {
        // CHECKS: Validate all conditions before making any state changes
        let liquidity_pool = &ctx.accounts.liquidity_pool;
        
        // Verify token mints match the pool configuration
        require!(
            ctx.accounts.token_x_mint.key() == liquidity_pool.token_x_mint &&
            ctx.accounts.token_y_mint.key() == liquidity_pool.token_y_mint,
            ErrorCode::InvalidToken
        );
        
        // Authorization check
        require!(
            ctx.accounts.authority.key() == liquidity_pool.super_admin ||
            ctx.accounts.authority.key() == liquidity_pool.authority,
            ErrorCode::Unauthorized
        );
        
        // Emptiness checks
        require!(
            liquidity_pool.token_x_reserve == 0 &&
            liquidity_pool.token_y_reserve == 0,
            ErrorCode::PoolNotEmpty
        );
        require!(
            liquidity_pool.lp_token_supply == 0,
            ErrorCode::PoolNotEmpty
        );
    
        // EFFECTS: Modify program state before any external interactions
        // Prepare to close the account and return lamports
        let current_balance = liquidity_pool.to_account_info().lamports();
        
        // Zero out the pool's balance first
        **liquidity_pool.to_account_info().try_borrow_mut_lamports()? = 0;
    
        // INTERACTIONS: Perform external account modifications
        // Transfer lamports to the authority
        **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += current_balance;
        
        emit!(PoolRemovedEvent {
            pool: liquidity_pool.key(),
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp
        });
        
        Ok(())
    }
    
    pub fn manage_admin(
        ctx: Context<ManageAdmin>,
        admin_address: Pubkey,
        is_add: bool,
    ) -> Result<()> {
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    
        // Ensure only super admin can modify admin list
        require!(
            ctx.accounts.authority.key() == liquidity_pool.super_admin,
            ErrorCode::Unauthorized
        );
    
        if is_add {
            // Adding an admin
            require!(
                !liquidity_pool.admins.contains(&admin_address),
                ErrorCode::AdminAlreadyExists
            );
            require!(
                liquidity_pool.admins.len() < 3,
                ErrorCode::MaxAdminLimitReached
            );
            liquidity_pool.admins.push(admin_address);
        } else {
            // Removing an admin
            require!(
                liquidity_pool.admins.contains(&admin_address),
                ErrorCode::AdminDoesntExist
            );
            liquidity_pool.admins.retain(|&x| x != admin_address);
        }
    
        // Emit event for admin changes
        emit!(AdminUpdated {
            admin: admin_address,
            is_added: is_add,
            super_admin: ctx.accounts.authority.key(),
        });
    
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<ProvideLiquidity>,
        token_x_mint: Pubkey,
        token_y_mint: Pubkey,
        amount_x: u64,
        amount_y: u64,
    ) -> Result<()> {
        // CHECKS
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        
        // Validate input amounts
        require!(amount_x > 0 && amount_y > 0, ErrorCode::InvalidLiquidityAmount);
        
        // Verify token mints
        require!(
            token_x_mint == ctx.accounts.token_x_mint.key(),
            ErrorCode::InvalidToken
        );
        require!(
            token_y_mint == ctx.accounts.token_y_mint.key(),
            ErrorCode::InvalidToken
        );
        
        // Verify balances
        require!(
            ctx.accounts.user_token_x_account.amount >= amount_x,
            ErrorCode::InsufficientFunds
        );
        require!(
            ctx.accounts.user_token_y_account.amount >= amount_y,
            ErrorCode::InsufficientFunds
        );
        
        // Verify ratio
        if liquidity_pool.token_x_reserve > 0 {
            let expected_ratio = (amount_y as u128)
                .checked_mul(liquidity_pool.token_x_reserve as u128)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(liquidity_pool.token_y_reserve as u128)
                .ok_or(ErrorCode::MathOverflow)?;
            require!(
                (amount_x as u128) == expected_ratio,
                ErrorCode::InvalidTokenRatio
            );
        }
    
        // Calculate LP tokens
        let lp_tokens = liquidity_pool.calculate_lp_tokens(amount_x, amount_y)?;
    
        // EFFECTS
        // Update reserves
        liquidity_pool.token_x_reserve = liquidity_pool.token_x_reserve
            .checked_add(amount_x)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.token_y_reserve = liquidity_pool.token_y_reserve
            .checked_add(amount_y)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.lp_token_supply = liquidity_pool.lp_token_supply
            .checked_add(lp_tokens)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // Record LP tokens
        liquidity_pool.lp_tokens.push(LpTokenBalance {
            owner: ctx.accounts.user.key(),
            amount: lp_tokens,
            last_reward_claim: Clock::get()?.unix_timestamp,
        });
    
        // INTERACTIONS
        // Transfer token X
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_x_account.to_account_info(),
                    to: ctx.accounts.pool_token_x_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.token_x_mint.to_account_info(),
                },
            ),
            amount_x,
            ctx.accounts.token_x_mint.decimals,
        )?;
    
        // Transfer token Y
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_y_account.to_account_info(),
                    to: ctx.accounts.pool_token_y_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.token_y_mint.to_account_info(),
                },
            ),
            amount_y,
            ctx.accounts.token_y_mint.decimals,
        )?;
    
        // Emit event
        emit!(LiquidityProvided {
            user: ctx.accounts.user.key(),
            token_x_amount: amount_x,
            token_y_amount: amount_y,
            lp_tokens_minted: lp_tokens,
        });
    
        Ok(())
    }
    
    pub fn swap_tokens(
        ctx: Context<SwapTokens>,
        input_token: Pubkey,
        output_token: Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        // CHECKS
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        
        // Input validation
        require!(amount_in > 0, ErrorCode::InvalidSwapInput);
        require!(minimum_amount_out > 0, ErrorCode::InvalidSwapInput);
        require!(input_token != output_token, ErrorCode::InvalidTokenPair);
        
        // Pool token validation
        require!(
            input_token == liquidity_pool.token_x_mint || input_token == liquidity_pool.token_y_mint,
            ErrorCode::InvalidToken
        );
        require!(
            output_token == liquidity_pool.token_x_mint || output_token == liquidity_pool.token_y_mint,
            ErrorCode::InvalidToken
        );
    
        // Determine swap direction and reserves
        let is_input_token_x = input_token == ctx.accounts.token_x_mint.key();
        let (input_reserve, output_reserve) = if is_input_token_x {
            (liquidity_pool.token_x_reserve, liquidity_pool.token_y_reserve)
        } else {
            (liquidity_pool.token_y_reserve, liquidity_pool.token_x_reserve)
        };
    
        // Calculate amounts
        let total_fee_amount = (amount_in as u128)
            .checked_mul(TOTAL_FEE_RATE as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::MathOverflow)? as u64;
    
        let amount_in_after_fees = amount_in
            .checked_sub(total_fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    
        let output_amount = calculate_swap_output(
            amount_in_after_fees,
            input_reserve,
            output_reserve,
        )?;
    
        require!(
            output_amount >= minimum_amount_out,
            ErrorCode::ExcessiveSlippage
        );
    
        // EFFECTS
        if is_input_token_x {
            liquidity_pool.token_x_reserve = liquidity_pool.token_x_reserve
                .checked_add(amount_in)
                .ok_or(ErrorCode::MathOverflow)?;
            liquidity_pool.token_y_reserve = liquidity_pool.token_y_reserve
                .checked_sub(output_amount)
                .ok_or(ErrorCode::MathOverflow)?;
        } else {
            liquidity_pool.token_y_reserve = liquidity_pool.token_y_reserve
                .checked_add(amount_in)
                .ok_or(ErrorCode::MathOverflow)?;
            liquidity_pool.token_x_reserve = liquidity_pool.token_x_reserve
                .checked_sub(output_amount)
                .ok_or(ErrorCode::MathOverflow)?;
        }
    
        // INTERACTIONS
        // Transfer input tokens
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_in.to_account_info(),
                    to: if is_input_token_x {
                        ctx.accounts.pool_token_x.to_account_info()
                    } else {
                        ctx.accounts.pool_token_y.to_account_info()
                    },
                    authority: ctx.accounts.user.to_account_info(),
                    mint: if is_input_token_x {
                        ctx.accounts.token_x_mint.to_account_info()
                    } else {
                        ctx.accounts.token_y_mint.to_account_info()
                    },
                },
            ),
            amount_in,
            if is_input_token_x {
                ctx.accounts.token_x_mint.decimals
            } else {
                ctx.accounts.token_y_mint.decimals
            },
        )?;
    
        // Transfer output tokens
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_out.to_account_info(),
                    to: if is_input_token_x {
                        ctx.accounts.pool_token_y.to_account_info()
                    } else {
                        ctx.accounts.pool_token_x.to_account_info()
                    },
                    authority: ctx.accounts.user.to_account_info(),
                    mint: if is_input_token_x {
                        ctx.accounts.token_y_mint.to_account_info()
                    } else {
                        ctx.accounts.token_x_mint.to_account_info()
                    },
                },
            ),
            output_amount,
            if is_input_token_x {
                ctx.accounts.token_y_mint.decimals
            } else {
                ctx.accounts.token_x_mint.decimals
            },
        )?;
    
        emit!(TokensSwapped {
            input_token: input_token.to_string(),
            input_amount: amount_in,
            output_amount,
        });
    
        Ok(())
    }
    
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        token_x_mint: Pubkey,
        token_y_mint: Pubkey,
        amount_x: u64,
        amount_y: u64,
    ) -> Result<()> {
        // CHECKS
        // Verify token mints
        require!(
            token_x_mint == ctx.accounts.token_x_mint.key(),
            ErrorCode::InvalidToken
        );
        require!(
            token_y_mint == ctx.accounts.token_y_mint.key(),
            ErrorCode::InvalidToken
        );
    
        let token_x_key = ctx.accounts.token_x_mint.key();
        let token_y_key = ctx.accounts.token_y_mint.key();
        let bump = ctx.accounts.liquidity_pool.bump;
    
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        let lp_tokens = liquidity_pool.calculate_lp_tokens(amount_x, amount_y)?;
    
        // Find and verify user LP balance
        let user_lp_index = liquidity_pool.lp_tokens
            .iter()
            .position(|x| x.owner == ctx.accounts.user.key())
            .ok_or(ErrorCode::NoLiquidity)?;
    
        // Verify sufficient reserves
        let new_token_x_reserve = liquidity_pool.token_x_reserve
            .checked_sub(amount_x)
            .ok_or(ErrorCode::MathOverflow)?;
        let new_token_y_reserve = liquidity_pool.token_y_reserve
            .checked_sub(amount_y)
            .ok_or(ErrorCode::MathOverflow)?;
        let new_lp_supply = liquidity_pool.lp_token_supply
            .checked_sub(lp_tokens)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // EFFECTS
        {
            // Update pool reserves
            liquidity_pool.token_x_reserve = new_token_x_reserve;
            liquidity_pool.token_y_reserve = new_token_y_reserve;
            liquidity_pool.lp_token_supply = new_lp_supply;
            
            // Update user LP balance
            let user_lp_balance = &mut liquidity_pool.lp_tokens[user_lp_index];
            require!(user_lp_balance.amount >= lp_tokens, ErrorCode::InsufficientFunds);
            user_lp_balance.amount = user_lp_balance.amount
                .checked_sub(lp_tokens)
                .ok_or(ErrorCode::MathOverflow)?;
    
            // Remove user if no remaining LP tokens
            if user_lp_balance.amount == 0 {
                liquidity_pool.lp_tokens.remove(user_lp_index);
            }
        }
    
        // INTERACTIONS
        let pool_seeds = &[POOL_SEED, token_x_key.as_ref(), token_y_key.as_ref(), &[bump]];
        let liquidity_pool_info = liquidity_pool.to_account_info();
    
        // Return token X to user
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_x_account.to_account_info(),
                    to: ctx.accounts.user_token_x_account.to_account_info(),
                    authority: liquidity_pool_info.clone(),
                    mint: ctx.accounts.token_x_mint.to_account_info(),
                },
                &[pool_seeds],
            ),
            amount_x,
            ctx.accounts.token_x_mint.decimals,
        )?;
    
        // Return token Y to user
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.pool_token_y_account.to_account_info(),
                    to: ctx.accounts.user_token_y_account.to_account_info(),
                    authority: liquidity_pool_info,
                    mint: ctx.accounts.token_y_mint.to_account_info(),
                },
                &[pool_seeds],
            ),
            amount_y,
            ctx.accounts.token_y_mint.decimals,
        )?;
    
        emit!(LiquidityRemoved {
            user: ctx.accounts.user.key(),
            token_x_amount: amount_x,
            token_y_amount: amount_y,
            lp_tokens_burned: lp_tokens,
        });
    
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    // CHECKS
    let token_x_key = ctx.accounts.token_x_mint.key();
    let token_y_key = ctx.accounts.token_y_mint.key();
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;

    // Verify user LP balance exists
    let user_lp_index = liquidity_pool.lp_tokens
        .iter()
        .position(|x| x.owner == ctx.accounts.user.key())
        .ok_or(ErrorCode::NoLiquidity)?;

    let user_lp_balance = &liquidity_pool.lp_tokens[user_lp_index];
    
    // Calculate and verify rewards
    let current_timestamp = Clock::get()?.unix_timestamp;
    let rewards = calculate_rewards(
        user_lp_balance.amount,
        liquidity_pool.reward_rate,
        current_timestamp - user_lp_balance.last_reward_claim
    )?;
    require!(rewards > 0, ErrorCode::NoRewardsAvailable);

    // EFFECTS
    liquidity_pool.lp_tokens[user_lp_index].last_reward_claim = current_timestamp;

    // INTERACTIONS
    let pool_seeds = &[
        POOL_SEED,
        token_x_key.as_ref(),
        token_y_key.as_ref(),
        &[liquidity_pool.bump],
    ];

    // Transfer rewards
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.pool_reward_account.to_account_info(),
                to: ctx.accounts.user_reward_account.to_account_info(),
                authority: liquidity_pool.to_account_info(),
                mint: ctx.accounts.reward_mint.to_account_info(),
            },
            &[pool_seeds],
        ),
        rewards,
        ctx.accounts.reward_mint.decimals,
    )?;

    emit!(RewardsClaimed {
        user: ctx.accounts.user.key(),
        amount: rewards,
        timestamp: current_timestamp,
    });

    Ok(())
}
    
    

    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
    ) -> Result<()> {
        // CHECKS
        require!(amount > 0, ErrorCode::InvalidStakeAmount);
        
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        let user_state = &mut ctx.accounts.user_state;
    
        // Calculate pending rewards before updating stake
        let current_timestamp = Clock::get()?.unix_timestamp;
        if user_state.amount_staked > 0 {
            let pending_rewards = calculate_rewards(
                user_state.amount_staked,
                liquidity_pool.reward_rate,
                current_timestamp - user_state.last_stake_timestamp
            )?;
            user_state.rewards_earned = user_state.rewards_earned
                .checked_add(pending_rewards)
                .ok_or(ErrorCode::MathOverflow)?;
        }
    
        // EFFECTS
        user_state.amount_staked = user_state.amount_staked
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        user_state.last_stake_timestamp = current_timestamp;
        
        liquidity_pool.total_staked = liquidity_pool.total_staked
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // INTERACTIONS
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                }
            ),
            amount,
            ctx.accounts.token_mint.decimals,
        )?;
    
        emit!(TokensStaked {
            user: ctx.accounts.user.key(),
            amount,
            timestamp: current_timestamp,
        });
    
        Ok(())
    }
    
    pub fn unstake(
    ctx: Context<Unstake>,
    amount: u64,
) -> Result<()> {
    // CHECKS
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let user_state = &mut ctx.accounts.user_state;
    
    // Store token keys for later use
    let token_x_key = ctx.accounts.token_x_mint.key();
    let token_y_key = ctx.accounts.token_y_mint.key();
    
    require!(amount > 0, ErrorCode::InvalidStakeAmount);
    require!(user_state.amount_staked >= amount, ErrorCode::InsufficientStake);

    // Calculate rewards
    let current_timestamp = Clock::get()?.unix_timestamp;
    let stake_duration = current_timestamp - user_state.last_stake_timestamp;
    let rewards = calculate_rewards(amount, liquidity_pool.reward_rate, stake_duration)?;
    let total_withdrawal = amount
        .checked_add(rewards)
        .ok_or(ErrorCode::MathOverflow)?;

    // EFFECTS
    user_state.amount_staked = user_state.amount_staked
        .checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    user_state.last_stake_timestamp = current_timestamp;
    
    liquidity_pool.total_staked = liquidity_pool.total_staked
        .checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    // INTERACTIONS
    let pool_seeds = &[
        POOL_SEED,
        token_x_key.as_ref(),
        token_y_key.as_ref(),
        &[liquidity_pool.bump],
    ];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.pool_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: liquidity_pool.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
            },
            &[pool_seeds],
        ),
        total_withdrawal,
        ctx.accounts.token_mint.decimals,
    )?;

    emit!(TokensUnstaked {
        user: ctx.accounts.user.key(),
        amount,
        rewards,
        timestamp: current_timestamp,
    });

    Ok(())
}
    
}



#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + LiquidityPool::INIT_SPACE,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub pool_token_x_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)] 
    pub pool_token_y_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: This is safe because we only use it to transfer SOL as protocol fee
    #[account(mut)]
    pub protocol_wallet: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct RemovePool<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
            ],
            bump = liquidity_pool.bump,
            close = authority
        )]
        pub liquidity_pool: Account<'info, LiquidityPool>,
        
        #[account(mut)]
        pub authority: Signer<'info>,
        
        pub token_x_mint: InterfaceAccount<'info, Mint>,
        pub token_y_mint: InterfaceAccount<'info, Mint>,
 }

#[derive(Accounts)]
pub struct ManageAdmin<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub authority: Signer<'info>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
}

#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, amount_x: u64, amount_y: u64)]
pub struct ProvideLiquidity<'info> {
    #[account(mut)]
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub token_y_mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_x_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_y_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_x_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_y_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, amount_x: u64, amount_y: u64)]
pub struct RemoveLiquidity<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_x_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)] 
    pub user_token_y_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_x_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_y_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
}



#[derive(Accounts)]
#[instruction(input_token: Pubkey, output_token: Pubkey, amount_in: u64, minimum_amount_out: u64)]
pub struct SwapTokens<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_in: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_out: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_x: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_y: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
}


#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserState::INIT_SPACE,
        seeds = [
            b"user_state",
            liquidity_pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserState::INIT_SPACE,
        seeds = [
            b"user_state",
            liquidity_pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        mut,
        seeds = [
            b"user_state",
            liquidity_pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_reward_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub pool_reward_account: InterfaceAccount<'info, TokenAccount>,
    pub reward_mint: InterfaceAccount<'info, Mint>,
    pub token_x_mint: InterfaceAccount<'info, Mint>,
    pub token_y_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub authority: Pubkey,
    pub super_admin: Pubkey,
    #[max_len(3)]
    pub admins: Vec<Pubkey>,
    pub token_x_reserve: u64,
    pub token_y_reserve: u64,
    pub lp_token_supply: u64,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    #[max_len(100)]
    pub lp_tokens: Vec<LpTokenBalance>,
    pub fee_rate: u64,
    pub bump: u8,
    pub reward_rate: u64,
    pub total_staked: u64,
}

#[account]
#[derive(InitSpace)]
pub struct UserState {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub last_stake_timestamp: i64,
    pub rewards_earned: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, InitSpace)]
pub struct LpTokenBalance {
    pub owner: Pubkey,
    pub amount: u64,
    pub last_reward_claim: i64,
}


#[event]
pub struct PoolInitialized {
    pub authority: Pubkey,
    pub fee_rate: u64,
}

#[event]
pub struct PoolRemovedEvent {
    pool: Pubkey,
    authority: Pubkey,
    timestamp: i64,
}

#[event]
pub struct AdminUpdated {
    pub admin: Pubkey,
    pub is_added: bool,
    pub super_admin: Pubkey,
}

#[event]
pub struct LiquidityProvided {
    pub user: Pubkey,
    pub token_x_amount: u64,
    pub token_y_amount: u64,
    pub lp_tokens_minted: u64,
}

#[event]
pub struct LiquidityRemoved {
    pub user: Pubkey,
    pub token_x_amount: u64,
    pub token_y_amount: u64,
    pub lp_tokens_burned: u64,
}

#[event]
pub struct TokensSwapped {
    pub input_token: String,
    pub input_amount: u64,
    pub output_amount: u64,
}

#[event]
pub struct TokensStaked {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensUnstaked {
    pub user: Pubkey,
    pub amount: u64,
    pub rewards: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {

    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Admin already exists")]
    AdminAlreadyExists,

    #[msg("Admin doesn't exists")]
    AdminDoesntExist,
    
    #[msg("Maximum admin limit reached")]
    MaxAdminLimitReached,

    #[msg("Invalid token input")]
    InvalidToken,
    
    #[msg("Invalid Token Pair")]
    InvalidTokenPair,
    
    #[msg("Mathematical operation overflow")]
    MathOverflow,
    
    #[msg("Invalid liquidity amount")]
    InvalidLiquidityAmount,
    
    #[msg("Invalid swap input")]
    InvalidSwapInput,
    
    #[msg("Slippage tolerance exceeded")]
    ExcessiveSlippage,
    
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("Invalid fee rate")]
    InvalidFeeRate,
    
    #[msg("Invalid token ratio")]
    InvalidTokenRatio,

    #[msg("Invalid stake amount")]
    InvalidStakeAmount,
    
    #[msg("Insufficient stake balance")]
    InsufficientStake,
    
    #[msg("Invalid LP token amount")]
    InvalidLPTokenAmount,
    
    #[msg("No liquidity provided")]
    NoLiquidity,

    #[msg("No rewards available to claim")]
    NoRewardsAvailable,

    #[msg("Pool is not empty")]
    PoolNotEmpty,
}

impl LiquidityPool {
    pub fn calculate_lp_tokens(&self, token_x_amount: u64, token_y_amount: u64) -> Result<u64> {
        if self.lp_token_supply == 0 {
            Ok(((token_x_amount as u128)
                .checked_mul(token_y_amount as u128)
                .ok_or(ErrorCode::MathOverflow)?)
            .integer_sqrt() as u64)
        } else {
            let x_ratio = (token_x_amount as u128)
                .checked_mul(self.lp_token_supply as u128)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(self.token_x_reserve as u128)
                .ok_or(ErrorCode::MathOverflow)? as u64;

            let y_ratio = (token_y_amount as u128)
                .checked_mul(self.lp_token_supply as u128)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(self.token_y_reserve as u128)
                .ok_or(ErrorCode::MathOverflow)? as u64;

            Ok(std::cmp::min(x_ratio, y_ratio))
        }
    }
}

fn calculate_swap_output(amount_in: u64, reserve_in: u64, reserve_out: u64) -> Result<u64> {
    require!(amount_in > 0, ErrorCode::InvalidSwapInput);
    require!(
        reserve_in > 0 && reserve_out > 0,
        ErrorCode::InvalidSwapInput
    );

    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;

    let k = reserve_in
        .checked_mul(reserve_out)
        .ok_or(ErrorCode::MathOverflow)?;

    let new_reserve_in = reserve_in
        .checked_add(amount_in)
        .ok_or(ErrorCode::MathOverflow)?;

    let new_reserve_out = k
        .checked_div(new_reserve_in)
        .ok_or(ErrorCode::MathOverflow)?;

    let amount_out = reserve_out
        .checked_sub(new_reserve_out)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(amount_out > 0, ErrorCode::InvalidSwapInput);
    require!(amount_out <= reserve_out, ErrorCode::InsufficientFunds);

    Ok(amount_out.try_into().map_err(|_| ErrorCode::MathOverflow)?)
}

fn calculate_rewards(amount_staked: u64, reward_rate: u64, duration: i64) -> Result<u64> {
    if duration <= 0 {
        return Ok(0);
    }

    let reward = (amount_staked as u128)
        .checked_mul(reward_rate as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(duration as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::MathOverflow)? as u64;

    Ok(reward)
}


