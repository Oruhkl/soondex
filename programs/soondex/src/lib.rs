use crate::associated_token::AssociatedToken;
use anchor_lang::prelude::borsh;
use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::token::{self, Token, Transfer};
use integer_sqrt::IntegerSquareRoot;

// Pool constants
pub const POOL_SEED: &[u8] = b"pool";
pub const MAX_FEE_RATE: u64 = 10000; // 100% in basis points
pub const MAX_REWARD_RATE: u64 = 1000; // 10% in basis points
pub const SECONDS_IN_DAY: i64 = 24 * 60 * 60;
pub const SECONDS_IN_YEAR: i64 = 365 * SECONDS_IN_DAY;
pub const TOTAL_FEE_RATE: u64 = 25; // 0.25% in basis points
pub const LP_FEE_RATE: u64 = 22;    // 0.22% in basis points
pub const STAKER_FEE_RATE: u64 = 3; // 0.03% in basis points
pub const PROTOCOL_FEE_LAMPORTS: u64 = 150_000_000; // 0.15 SOL in lamports

// Liquidity constants
pub const IMBALANCE_THRESHOLD: u64 = 2;
pub const DIMINISHING_RETURNS_RATE: u128 = 90; // 90% for stakes over 1 year

// Math constants
pub const BASIS_POINTS_DIVISOR: u64 = 10000;

declare_id!("B4xt3vAan4S5UmUgucsxMPi2uwqEmrSSdvJnzVPWeUFu");

#[program]
pub mod soondex {
    use super::*;


    pub fn manage_admin(
        ctx: Context<ManageAdmin>,
        admin_address: Pubkey,
        is_add: bool,
    ) -> Result<()> {
        // CHECKS
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        
        // Verify super admin authority
        require!(
            ctx.accounts.authority.key() == liquidity_pool.super_admin,
            ErrorCode::Unauthorized
        );
        
        // Verify admin isn't already in list when adding
        if is_add {
            require!(
                !liquidity_pool.admins.contains(&admin_address),
                ErrorCode::AdminAlreadyExists
            );
        }
    
        // EFFECTS
        // Modify admin list
        if is_add {
            liquidity_pool.admins.push(admin_address);
        } else {
            liquidity_pool.admins.retain(|&x| x != admin_address);
        }
    
        // INTERACTIONS
        // Emit event for external notification
        emit!(AdminUpdated {
            admin: admin_address,
            is_added: is_add,
            super_admin: ctx.accounts.authority.key(),
        });
    
        Ok(())
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        token_x_mint: Pubkey,
        token_y_mint: Pubkey,
        fee_rate: u64,
        reward_rate: u64,
    ) -> Result<()> {
        // CHECKS
        //Validate input rates and tokens
        require!(fee_rate <= MAX_FEE_RATE, ErrorCode::InvalidFeeRate);
        require!(reward_rate <= MAX_REWARD_RATE, ErrorCode::InvalidRewardRate);
        require!(
            ctx.accounts.token_x_mint.key() == token_x_mint,
            ErrorCode::InvalidToken
        );
        require!(
            ctx.accounts.payer.lamports() >= PROTOCOL_FEE_LAMPORTS,
            ErrorCode::InsufficientFunds
        );
        
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        liquidity_pool.bump = ctx.bumps.liquidity_pool;

        // EFFECTS
        // 1. Initialize pool state
        liquidity_pool.authority = ctx.accounts.payer.key();  // Set authority as the payer
        liquidity_pool.fee_rate = fee_rate;
        liquidity_pool.reward_rate = reward_rate;
        liquidity_pool.total_staked = 0;
        liquidity_pool.token_x_reserve = 0;
        liquidity_pool.token_y_reserve = 0;
        liquidity_pool.lp_token_supply = 0;
        liquidity_pool.order_count = 0;
        liquidity_pool.lp_tokens = Vec::new();
        liquidity_pool.super_admin = ctx.accounts.payer.key();  // Set the payer as super admin

    
        // 2. Set PDA bump
        let (_pda, _bump) = Pubkey::find_program_address(
            &[POOL_SEED, ctx.accounts.token_x_mint.key().as_ref(), ctx.accounts.token_y_mint.key().as_ref()],
            ctx.program_id,
        );
    
        // INTERACTIONS
        // 1. Protocol fee collection
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
    
        // 2. Create associated token accounts
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
    
        // EVENTS
        emit!(PoolInitialized {
            authority: liquidity_pool.authority,
            fee_rate,
            reward_rate,
        });
    
        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<ProvideLiquidity>, 
        amount_x: u64,
        amount_y: u64,
        min_lp_tokens: u64
    ) -> Result<()> {
        // CHECKS
        require!(amount_x > 0 && amount_y > 0, ErrorCode::InvalidLiquidityAmount);
        
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        
        // Calculate LP tokens to mint
        let lp_tokens = liquidity_pool.calculate_lp_tokens(amount_x, amount_y)?;
        require!(lp_tokens >= min_lp_tokens, ErrorCode::ExcessiveSlippage);
    
        // EFFECTS
        // Update pool reserves
        liquidity_pool.token_x_reserve = liquidity_pool.token_x_reserve.checked_add(amount_x)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.token_y_reserve = liquidity_pool.token_y_reserve.checked_add(amount_y)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.lp_token_supply = liquidity_pool.lp_token_supply.checked_add(lp_tokens)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // Add LP token balance for user
        liquidity_pool.lp_tokens.push(LpTokenBalance {
            owner: ctx.accounts.user.key(),
            amount: lp_tokens,
        });
    
        // INTERACTIONS
        // Transfer tokens from user to pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_x_account.to_account_info(),
                    to: ctx.accounts.pool_token_x_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_x,
        )?;
    
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_y_account.to_account_info(),
                    to: ctx.accounts.pool_token_y_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_y,
        )?;
    
        // EVENTS
        emit!(LiquidityProvided {
            user: ctx.accounts.user.key(),
            token_x_amount: amount_x,
            token_y_amount: amount_y,
            lp_tokens_minted: lp_tokens,
        });
    
        Ok(())
    }
    
        
    
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        // CHECKS
        // 1. Validate input amount
        require!(amount > 0, ErrorCode::InvalidStakeAmount);
        
        // 2. Validate user has sufficient funds
        require!(
            ctx.accounts.user_token_account.amount >= amount,
            ErrorCode::InsufficientFunds
        );
        
        // 3. Validate token account ownership
        require!(
            ctx.accounts.user_token_account.owner == ctx.accounts.user.key(),
            ErrorCode::InvalidTokenOwner
        );
        
        // 4. Validate pool capacity
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        let new_total = liquidity_pool
            .total_staked
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // EFFECTS
        // 1. Update user state
        let user = &mut ctx.accounts.user_state;
        user.amount_staked = user
            .amount_staked
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        user.last_stake_timestamp = Clock::get()?.unix_timestamp;
    
        // 2. Update pool state
        liquidity_pool.total_staked = new_total;
    
        // INTERACTIONS
        // Transfer tokens from user to pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
    
        // EVENTS
        emit!(TokensStaked {
            user: ctx.accounts.user.key(),
            amount,
        });
    
        Ok(())
    }
    
    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>) -> Result<()> {
        // CHECKS
        // 1. Validate stake amount
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        let user = &mut ctx.accounts.user_state;
        require!(user.amount_staked > 0, ErrorCode::InvalidStakeAmount);
    
        // 2. Calculate withdrawal amounts
        let current_timestamp = Clock::get()?.unix_timestamp;
        let stake_duration = current_timestamp - user.last_stake_timestamp;
        let amount_staked = user.amount_staked;
        
        // Calculate rewards
        let rewards = calculate_rewards(amount_staked, liquidity_pool.reward_rate, stake_duration)?;
        let total_amount = amount_staked
            .checked_add(rewards)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // 3. Verify pool liquidity
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        require!(
            ctx.accounts.pool_token_account.amount >= total_amount,
            ErrorCode::InsufficientFunds
        );
    
        // EFFECTS
        // 1. Update pool state
        liquidity_pool.total_staked = liquidity_pool
            .total_staked
            .checked_sub(amount_staked)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // 2. Reset user state
        user.amount_staked = 0;
        user.last_stake_timestamp = 0;
    
        // INTERACTIONS
        // Transfer tokens and rewards to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(), 
                    authority: liquidity_pool.to_account_info(),
                },
                &[&[
                    POOL_SEED,
                    ctx.accounts.token_x_mint.key().as_ref(),
                    ctx.accounts.token_y_mint.key().as_ref(),
                    &[liquidity_pool.bump]
                ]],
            ),
            total_amount,
        )?;
        
    
        // EVENTS
        emit!(TokensWithdrawn {
            user: ctx.accounts.user.key(),
            amount: amount_staked,
            rewards,
        });
    
        Ok(())
    }
    
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        side: OrderSide,
        amount: u64,
        price: u64,
    ) -> Result<()> {
        // CHECKS
        // 1. Validate input parameters
        require!(amount > 0, ErrorCode::InvalidOrderAmount);
        require!(price > 0, ErrorCode::InvalidPrice);
        
        // 2. Verify user has sufficient funds
        require!(
            ctx.accounts.user_token_account.amount >= amount,
            ErrorCode::InsufficientFunds
        );
    
        // EFFECTS
        // 1. Create new order
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        let order = Order {
            id: liquidity_pool.order_count,
            owner: ctx.accounts.user.key(),
            side,
            amount,
            price,
            fulfilled: 0,
        };
    
        // 2. Update pool state
        liquidity_pool.orders.push(order);
        liquidity_pool.order_count += 1;
    
        // INTERACTIONS
        // Transfer tokens to pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
    
        // EVENTS
        emit!(OrderPlaced {
            order_id: liquidity_pool.order_count - 1,
            owner: ctx.accounts.user.key(),
            side,
            amount,
            price,
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
        
        // 1. Validate basic parameters
        require!(amount_in > 0, ErrorCode::InvalidSwapInput);
        require!(minimum_amount_out > 0, ErrorCode::InvalidSwapInput);
    
        // 2. Validate token direction
        let is_input_token_x = input_token == ctx.accounts.token_x_mint.key();
        let is_output_token_y = output_token == ctx.accounts.token_y_mint.key();
        require!(
            is_input_token_x && is_output_token_y || !is_input_token_x && !is_output_token_y,
            ErrorCode::InvalidTokenPair
        );
    
        // 3. Get current reserves
        let (input_reserve, output_reserve) = if is_input_token_x {
            (liquidity_pool.token_x_reserve, liquidity_pool.token_y_reserve)
        } else {
            (liquidity_pool.token_y_reserve, liquidity_pool.token_x_reserve)
        };
    
        // 4. Calculate fees and output
        let total_fee_amount = (amount_in as u128)
            .checked_mul(TOTAL_FEE_RATE as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::MathOverflow)? as u64;
    
        let lp_fee_amount = (amount_in as u128)
            .checked_mul(LP_FEE_RATE as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::MathOverflow)? as u64;
    
        let staker_fee_amount = total_fee_amount
            .checked_sub(lp_fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    
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
        // 1. Update metrics if needed
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - liquidity_pool.last_volume_reset >= SECONDS_IN_DAY {
            liquidity_pool.volume_24h = 0;
            liquidity_pool.fees_24h = 0;
            liquidity_pool.last_volume_reset = current_time;
        }
    
        // 2. Update reserves
        if is_input_token_x {
            liquidity_pool.token_x_reserve = input_reserve
                .checked_add(amount_in)
                .ok_or(ErrorCode::MathOverflow)?;
            liquidity_pool.token_y_reserve = output_reserve
                .checked_sub(output_amount)
                .ok_or(ErrorCode::MathOverflow)?;
        } else {
            liquidity_pool.token_y_reserve = input_reserve
                .checked_add(amount_in)
                .ok_or(ErrorCode::MathOverflow)?;
            liquidity_pool.token_x_reserve = output_reserve
                .checked_sub(output_amount)
                .ok_or(ErrorCode::MathOverflow)?;
        }
    
        // 3. Update pool metrics
        liquidity_pool.staking_rewards = liquidity_pool
            .staking_rewards
            .checked_add(staker_fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.volume_24h = liquidity_pool
            .volume_24h
            .checked_add(amount_in)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.fees_24h = liquidity_pool
            .fees_24h
            .checked_add(total_fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.tvl_x = liquidity_pool.token_x_reserve;
        liquidity_pool.tvl_y = liquidity_pool.token_y_reserve;
    
        // INTERACTIONS
        // 1. Transfer input tokens
        let pool_token_in = if is_input_token_x {
            ctx.accounts.pool_token_x.to_account_info()
        } else {
            ctx.accounts.pool_token_y.to_account_info()
        };
        
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_in.to_account_info(),
                    to: pool_token_in,
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;
    
        // 2. Transfer output tokens
        let pool_token_out = if is_input_token_x {
            ctx.accounts.pool_token_y.to_account_info()
        } else {
            ctx.accounts.pool_token_x.to_account_info()
        };
    
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: pool_token_out,
                    to: ctx.accounts.user_token_out.to_account_info(),
                    authority: liquidity_pool.to_account_info(),
                },
                &[&[
                    POOL_SEED,
                    ctx.accounts.token_x_mint.key().as_ref(),
                    ctx.accounts.token_y_mint.key().as_ref(),
                    &[liquidity_pool.bump],
                ]],
            ),
            output_amount,
        )?;
    
        // Event
        emit!(TokensSwapped {
            input_token: input_token.to_string(),
            input_amount: amount_in,
            output_amount,
        });
    
        Ok(())
    }
    
    pub fn match_orders(ctx: Context<MatchOrders>) -> Result<()> {
        // CHECKS
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        require!(!liquidity_pool.orders.is_empty(), ErrorCode::InvalidOrderAmount);
    
        // PREPARATION
        // 1. Filter and sort buy/sell orders
        let mut buy_orders: Vec<_> = liquidity_pool
            .orders
            .iter()
            .filter(|o| o.side == OrderSide::Buy && o.amount > o.fulfilled)
            .collect();
        let mut sell_orders: Vec<_> = liquidity_pool
            .orders
            .iter()
            .filter(|o| o.side == OrderSide::Sell && o.amount > o.fulfilled)
            .collect();
    
        buy_orders.sort_by(|a, b| b.price.cmp(&a.price));  // Highest buy first
        sell_orders.sort_by(|a, b| a.price.cmp(&b.price)); // Lowest sell first
    
        // 2. Initialize matching variables
        let fee_rate = liquidity_pool.fee_rate;
        let mut matches = Vec::new();
        let mut total_matched_volume = 0u64;
    
        // MATCHING LOGIC
        for buy_order in &buy_orders {
            for sell_order in &sell_orders {
                // Break if no match possible
                if buy_order.price < sell_order.price {
                    break;
                }
    
                // Calculate remaining amounts
                let buy_remaining = buy_order
                    .amount
                    .checked_sub(buy_order.fulfilled)
                    .ok_or(ErrorCode::MathOverflow)?;
                let sell_remaining = sell_order
                    .amount
                    .checked_sub(sell_order.fulfilled)
                    .ok_or(ErrorCode::MathOverflow)?;
    
                if buy_remaining == 0 || sell_remaining == 0 {
                    continue;
                }
    
                // Calculate match details
                let match_amount = std::cmp::min(buy_remaining, sell_remaining);
                let fee_amount = (match_amount as u128)
                    .checked_mul(fee_rate as u128)
                    .ok_or(ErrorCode::MathOverflow)?
                    .checked_div(10000)
                    .ok_or(ErrorCode::MathOverflow)? as u64;
                let amount_after_fee = match_amount
                    .checked_sub(fee_amount)
                    .ok_or(ErrorCode::MathOverflow)?;
                let execution_price = buy_order.price
                    .checked_add(sell_order.price)
                    .ok_or(ErrorCode::MathOverflow)?
                    .checked_div(2)
                    .ok_or(ErrorCode::MathOverflow)?;
    
                // Record match
                matches.push((
                    buy_order.id,
                    sell_order.id,
                    amount_after_fee,
                    execution_price,
                    fee_amount,
                ));
    
                total_matched_volume = total_matched_volume
                    .checked_add(amount_after_fee)
                    .ok_or(ErrorCode::MathOverflow)?;
            }
        }
    
        // EFFECTS
        // 1. Process matches and update orders
        let mut total_fees = 0u64;
        let mut total_volume = 0u64;
    
        // Inside match_orders function, update the matches loop:

for (buy_id, sell_id, match_amount, match_price, fee_amount) in &matches {
    // Update buy order
    if let Some(buy_order) = liquidity_pool.orders.iter_mut().find(|o| o.id == *buy_id) {
        buy_order.fulfilled = buy_order
            .fulfilled
            .checked_add(*match_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    }

    // Update sell order
    if let Some(sell_order) = liquidity_pool.orders.iter_mut().find(|o| o.id == *sell_id) {
        sell_order.fulfilled = sell_order
            .fulfilled
            .checked_add(*match_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    }

    total_fees = total_fees
        .checked_add(*fee_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    total_volume = total_volume
        .checked_add(*match_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    // Emit individual match event
    emit!(OrdersMatched {
        buy_order_id: *buy_id,
        sell_order_id: *sell_id,
        match_amount: *match_amount,
        match_price: *match_price,
    });
}

    
        // 2. Update pool state
        liquidity_pool.token_x_reserve = liquidity_pool
            .token_x_reserve
            .checked_add(total_fees)
            .ok_or(ErrorCode::MathOverflow)?;
        liquidity_pool.volume_24h = liquidity_pool
            .volume_24h
            .checked_add(total_volume)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // 3. Clean up matched orders
        liquidity_pool.orders.retain(|order| order.amount > order.fulfilled);
    
        // EVENTS
        if total_matched_volume > 0 {
            emit!(OrderMatchingComplete {
                total_matches: matches.len() as u64,
                total_volume: total_matched_volume,
            });
        }
    
        Ok(())
    }
    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u64) -> Result<()> {
        // CHECKS
        // 1. Find and validate order
        let liquidity_pool = &mut ctx.accounts.liquidity_pool;
        let order = liquidity_pool.orders
            .iter()
            .find(|o| o.id == order_id && o.owner == ctx.accounts.user.key())
            .ok_or(ErrorCode::OrderNotFound)?;
    
        // 2. Calculate refund amount
        let refund_amount = order.amount
            .checked_sub(order.fulfilled)
            .ok_or(ErrorCode::MathOverflow)?;
    
        // EFFECTS
        // 1. Remove order from pool
        liquidity_pool.orders.retain(|o| o.id != order_id);
    
        // INTERACTIONS
        // 1. Return unfulfilled tokens to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: liquidity_pool.to_account_info(),
                },
                &[&[
                    POOL_SEED,
                    ctx.accounts.token_x_mint.key().as_ref(),
                    ctx.accounts.token_y_mint.key().as_ref(),
                    &[liquidity_pool.bump]
                ]],
            ),
            refund_amount,
        )?;
    
        // EVENTS
        emit!(OrderCancelled {
            order_id,
            owner: ctx.accounts.user.key(),
            refund_amount,
        });
    
        Ok(())
    }
    
}    

#[derive(Accounts)]
pub struct ManageAdmin<'info> {
    #[account(mut)]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub authority: Signer<'info>,
}


#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64, reward_rate: u64)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<LiquidityPool>(),
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_x_mint: Box<Account<'info, token::Mint>>,
    pub token_y_mint: Box<Account<'info, token::Mint>>,

    #[account(mut)]
    pub pool_token_x_account: Box<Account<'info, token::TokenAccount>>,

    #[account(mut)]
    pub pool_token_y_account: Box<Account<'info, token::TokenAccount>>,
    /// CHECK: This is safe because we only use it to transfer SOL as protocol fee
    #[account(mut)]
    pub protocol_wallet: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64, reward_rate: u64)]
pub struct ProvideLiquidity<'info> {
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
    pub user_token_x_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub user_token_y_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_x_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_y_account: Box<Account<'info, token::TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_x_account: Account<'info, token::TokenAccount>,
    #[account(mut)] 
    pub user_token_y_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub pool_token_x_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub pool_token_y_account: Account<'info, token::TokenAccount>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64, reward_rate: u64)]
pub struct StakeTokens<'info> {
    #[account(mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserState>(),
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
    pub user_token_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub pool_token_account: Account<'info, token::TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey)]
pub struct WithdrawTokens<'info> {
    #[account(mut,
        seeds = [
            POOL_SEED,
            token_x_mint.key().as_ref(),
            token_y_mint.key().as_ref()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
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
    pub user_token_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_account: Box<Account<'info, token::TokenAccount>>,
    
    pub token_program: Program<'info, Token>,
    pub token_x_mint: Box<Account<'info, token::Mint>>,
    pub token_y_mint: Box<Account<'info, token::Mint>>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64, reward_rate: u64)]

pub struct PlaceOrder<'info> {
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
    pub user_token_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_account: Box<Account<'info, token::TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64, reward_rate: u64)]

pub struct MatchOrders<'info> {
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
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey, fee_rate: u64, reward_rate: u64)]

pub struct SwapTokens<'info> {
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
    pub user_token_in: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub user_token_out: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_in: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_x: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub pool_token_y: Account<'info, token::TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub token_x_mint: Account<'info, token::Mint>,
    pub token_y_mint: Account<'info, token::Mint>,
}
#[derive(Accounts)]
#[instruction(token_x_mint: Pubkey, token_y_mint: Pubkey)]
pub struct CancelOrder<'info> {
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
    pub user_token_account: Box<Account<'info, token::TokenAccount>>,
    #[account(mut)]
    pub pool_token_account: Box<Account<'info, token::TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub token_x_mint: Box<Account<'info, token::Mint>>,
    pub token_y_mint: Box<Account<'info, token::Mint>>,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LpTokenBalance {
    pub owner: Pubkey,
    pub amount: u64,
}


fn calculate_swap_output(amount_in: u64, reserve_in: u64, reserve_out: u64) -> Result<u64> {
    // Input validation
    require!(amount_in > 0, ErrorCode::InvalidSwapInput);
    require!(
        reserve_in > 0 && reserve_out > 0,
        ErrorCode::InvalidPoolState
    );

    // Convert to u128 for intermediate calculations to prevent overflow
    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;

    // Calculate constant product k = x * y
    let k = reserve_in
        .checked_mul(reserve_out)
        .ok_or(ErrorCode::MathOverflow)?;

    // Calculate new reserve after swap: (reserve_in + amount_in)
    let new_reserve_in = reserve_in
        .checked_add(amount_in)
        .ok_or(ErrorCode::MathOverflow)?;

    // Calculate new reserve_out using constant product formula:
    // k = new_reserve_in * new_reserve_out
    // new_reserve_out = k / new_reserve_in
    let new_reserve_out = k
        .checked_div(new_reserve_in)
        .ok_or(ErrorCode::MathOverflow)?;

    // Calculate output amount: reserve_out - new_reserve_out
    let amount_out = reserve_out
        .checked_sub(new_reserve_out)
        .ok_or(ErrorCode::MathOverflow)?;

    // Verify output amount is reasonable
    require!(amount_out > 0, ErrorCode::InvalidSwapInput);
    require!(amount_out <= reserve_out, ErrorCode::InsufficientFunds);

    // Convert back to u64 with overflow check
    Ok(amount_out.try_into().map_err(|_| ErrorCode::MathOverflow)?)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapParams {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Order {
    pub id: u64,
    pub owner: Pubkey,
    pub side: OrderSide,
    pub amount: u64,
    pub price: u64,
    pub fulfilled: u64,
}

#[account]
pub struct LiquidityPool {
    pub authority: Pubkey,
    pub admins: Vec<Pubkey>,
    pub super_admin: Pubkey, 
    pub token_x_reserve: u64,
    pub token_y_reserve: u64,
    pub lp_token_supply: u64,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub lp_tokens: Vec<LpTokenBalance>,
    pub fee_rate: u64,
    pub reward_rate: u64,
    pub total_staked: u64,
    pub bump: u8,
    pub order_count: u64,
    pub orders: Vec<Order>,
    pub volume_24h: u64,
    pub fees_24h: u64,
    pub last_volume_reset: i64,
    pub tvl_x: u64,
    pub tvl_y: u64,
    pub staking_rewards: u64
}


#[event]
pub struct AdminUpdated {
    pub admin: Pubkey,
    pub is_added: bool,
    pub super_admin: Pubkey,
}

#[account]
pub struct UserState {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub last_stake_timestamp: i64,
    pub rewards_earned: u64,
    pub last_reward_timestamp: i64,
    pub bump: u8,
}
#[event]
pub struct PoolInitialized {
    pub authority: Pubkey,
    pub fee_rate: u64,
    pub reward_rate: u64,
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
pub struct RewardsHarvested {
    pub user: Pubkey,
    pub amount: u64,
}


#[event]
pub struct TokensStaked {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TokensWithdrawn {
    pub user: Pubkey,
    pub amount: u64,
    pub rewards: u64,
}

#[event]
pub struct TokensSwapped {
    pub input_token: String,
    pub input_amount: u64,
    pub output_amount: u64,
}

#[event]
pub struct OrderPlaced {
    pub order_id: u64,
    pub owner: Pubkey,
    pub side: OrderSide,
    pub amount: u64,
    pub price: u64,
}

#[event]
pub struct OrdersMatched {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub match_amount: u64,
    pub match_price: u64,
}

#[event]
pub struct OrderMatchingComplete {
    pub total_matches: u64,
    pub total_volume: u64,
}

#[event]
pub struct OrderCancelled {
    pub order_id: u64,
    pub owner: Pubkey,
    pub refund_amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid token input")]
    InvalidToken,

    #[msg("Invalid Token Pair")]
    InvalidTokenPair,

    #[msg("Mathematical operation overflow")]
    MathOverflow,

    #[msg("Invalid liquidity amount")]
    InvalidLiquidityAmount,

    #[msg("Invalid stake amount")]
    InvalidStakeAmount,

    #[msg("Invalid swap input")]
    InvalidSwapInput,

    #[msg("Slippage tolerance exceeded")]
    ExcessiveSlippage,

    #[msg("Order not found")]
    OrderNotFound,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Invalid token owner")]
    InvalidTokenOwner,

    #[msg("Transfer not approved")]
    TransferNotApproved,

    #[msg("Unbalanced liquidity pool")]
    UnbalancedLiquidity,

    #[msg("Invalid fee rate")]
    InvalidFeeRate,

    #[msg("Invalid reward rate")]
    InvalidRewardRate,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("Invalid order amount")]
    InvalidOrderAmount,

    #[msg("Order limit exceeded")]
    OrderLimitExceeded,

    #[msg("Arithmetic Error")]
    ArithmeticError,

    #[msg("Invalid stake duration")]
    InvalidStakeDuration,

    #[msg("Pool capacity exceeded")]
    PoolCapacityExceeded,

    #[msg("Reward calculation error")]
    RewardCalculationError,

    #[msg("Invalid pool state")]
    InvalidPoolState,

    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Invalid Liquidity PoolPDA")]
    InvalidLiquidityPoolPDA,

    #[msg("No liquidity provided")]
    NoLiquidity,
    
    #[msg("Invalid LP token amount")]
    InvalidLPTokenAmount,
    
    #[msg("LP tokens locked")]
    LPTokensLocked,
    #[msg("Admin already exists")]
    AdminAlreadyExists,
    
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

    pub fn check_reserves(&self) -> Result<()> {
        if self.token_x_reserve > self.token_y_reserve * IMBALANCE_THRESHOLD {
            return Err(ErrorCode::UnbalancedLiquidity.into());
        }
        if self.token_y_reserve > self.token_x_reserve * IMBALANCE_THRESHOLD {
            return Err(ErrorCode::UnbalancedLiquidity.into());
        }
        Ok(())
    }
}

fn calculate_rewards(amount_staked: u64, reward_rate: u64, stake_duration: i64) -> Result<u64> {
    if stake_duration <= 0 {
        return Ok(0);
    }

    let duration = stake_duration as u64;

    // Calculate base reward: amount * rate * duration / 10000
    let base_reward = (amount_staked as u128)
        .checked_mul(reward_rate as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(duration as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::MathOverflow)?;

    // Apply diminishing returns for long stake durations
    let final_reward = if duration > SECONDS_IN_YEAR as u64 {
        base_reward
            .checked_mul(DIMINISHING_RETURNS_RATE)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(100)
            .ok_or(ErrorCode::MathOverflow)?
    } else {
        base_reward
    };

    Ok(final_reward
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?)
}