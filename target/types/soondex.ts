/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/soondex.json`.
 */
export type Soondex = {
  "address": "8BPmoTqwHpFGECrtmWbDSz8wDXj5X7e32qdkLrNhQVqy",
  "metadata": {
    "name": "soondex",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "addLiquidity",
      "discriminator": [
        181,
        157,
        89,
        67,
        143,
        182,
        52,
        72
      ],
      "accounts": [
        {
          "name": "tokenXMint",
          "writable": true
        },
        {
          "name": "tokenYMint",
          "writable": true
        },
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "arg",
                "path": "tokenXMint"
              },
              {
                "kind": "arg",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userTokenXAccount",
          "writable": true
        },
        {
          "name": "userTokenYAccount",
          "writable": true
        },
        {
          "name": "poolTokenXAccount",
          "writable": true
        },
        {
          "name": "poolTokenYAccount",
          "writable": true
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "tokenXMint",
          "type": "pubkey"
        },
        {
          "name": "tokenYMint",
          "type": "pubkey"
        },
        {
          "name": "amountX",
          "type": "u64"
        },
        {
          "name": "amountY",
          "type": "u64"
        }
      ]
    },
    {
      "name": "claimRewards",
      "discriminator": [
        4,
        144,
        132,
        71,
        116,
        23,
        151,
        80
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenXMint"
              },
              {
                "kind": "account",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "userState",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "liquidityPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userRewardAccount",
          "writable": true
        },
        {
          "name": "poolRewardAccount",
          "writable": true
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "initializePool",
      "discriminator": [
        95,
        180,
        10,
        172,
        84,
        174,
        232,
        40
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "arg",
                "path": "tokenXMint"
              },
              {
                "kind": "arg",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        },
        {
          "name": "poolTokenXAccount",
          "writable": true
        },
        {
          "name": "poolTokenYAccount",
          "writable": true
        },
        {
          "name": "protocolWallet",
          "writable": true
        }
      ],
      "args": [
        {
          "name": "tokenXMint",
          "type": "pubkey"
        },
        {
          "name": "tokenYMint",
          "type": "pubkey"
        },
        {
          "name": "feeRate",
          "type": "u64"
        }
      ]
    },
    {
      "name": "manageAdmin",
      "discriminator": [
        141,
        136,
        128,
        177,
        111,
        187,
        95,
        148
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenXMint"
              },
              {
                "kind": "account",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        }
      ],
      "args": [
        {
          "name": "adminAddress",
          "type": "pubkey"
        },
        {
          "name": "isAdd",
          "type": "bool"
        }
      ]
    },
    {
      "name": "removeLiquidity",
      "discriminator": [
        80,
        85,
        209,
        72,
        24,
        206,
        177,
        108
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "arg",
                "path": "tokenXMint"
              },
              {
                "kind": "arg",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userTokenXAccount",
          "writable": true
        },
        {
          "name": "userTokenYAccount",
          "writable": true
        },
        {
          "name": "poolTokenXAccount",
          "writable": true
        },
        {
          "name": "poolTokenYAccount",
          "writable": true
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        }
      ],
      "args": [
        {
          "name": "tokenXMint",
          "type": "pubkey"
        },
        {
          "name": "tokenYMint",
          "type": "pubkey"
        },
        {
          "name": "amountX",
          "type": "u64"
        },
        {
          "name": "amountY",
          "type": "u64"
        }
      ]
    },
    {
      "name": "removePool",
      "discriminator": [
        132,
        42,
        53,
        138,
        28,
        220,
        170,
        55
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenXMint"
              },
              {
                "kind": "account",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        }
      ],
      "args": [
        {
          "name": "tokenXMint",
          "type": "pubkey"
        },
        {
          "name": "tokenYMint",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "stake",
      "discriminator": [
        206,
        176,
        202,
        18,
        200,
        209,
        179,
        108
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenXMint"
              },
              {
                "kind": "account",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "userState",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "liquidityPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "poolTokenAccount",
          "writable": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "swapTokens",
      "discriminator": [
        201,
        226,
        234,
        16,
        70,
        155,
        131,
        206
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenXMint"
              },
              {
                "kind": "account",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userTokenIn",
          "writable": true
        },
        {
          "name": "userTokenOut",
          "writable": true
        },
        {
          "name": "poolTokenX",
          "writable": true
        },
        {
          "name": "poolTokenY",
          "writable": true
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        },
        {
          "name": "wsolAccount",
          "writable": true,
          "optional": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "nativeMint"
        }
      ],
      "args": [
        {
          "name": "inputToken",
          "type": "pubkey"
        },
        {
          "name": "outputToken",
          "type": "pubkey"
        },
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minimumAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "unstake",
      "discriminator": [
        90,
        95,
        107,
        42,
        205,
        124,
        50,
        225
      ],
      "accounts": [
        {
          "name": "liquidityPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "tokenXMint"
              },
              {
                "kind": "account",
                "path": "tokenYMint"
              }
            ]
          }
        },
        {
          "name": "userState",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  117,
                  115,
                  101,
                  114,
                  95,
                  115,
                  116,
                  97,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "liquidityPool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "poolTokenAccount",
          "writable": true
        },
        {
          "name": "tokenMint"
        },
        {
          "name": "tokenXMint"
        },
        {
          "name": "tokenYMint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "liquidityPool",
      "discriminator": [
        66,
        38,
        17,
        64,
        188,
        80,
        68,
        129
      ]
    },
    {
      "name": "userState",
      "discriminator": [
        72,
        177,
        85,
        249,
        76,
        167,
        186,
        126
      ]
    }
  ],
  "events": [
    {
      "name": "adminUpdated",
      "discriminator": [
        69,
        82,
        49,
        171,
        43,
        3,
        80,
        161
      ]
    },
    {
      "name": "liquidityProvided",
      "discriminator": [
        94,
        97,
        39,
        34,
        15,
        96,
        79,
        135
      ]
    },
    {
      "name": "liquidityRemoved",
      "discriminator": [
        225,
        105,
        216,
        39,
        124,
        116,
        169,
        189
      ]
    },
    {
      "name": "poolInitialized",
      "discriminator": [
        100,
        118,
        173,
        87,
        12,
        198,
        254,
        229
      ]
    },
    {
      "name": "poolRemovedEvent",
      "discriminator": [
        46,
        214,
        235,
        75,
        135,
        86,
        37,
        142
      ]
    },
    {
      "name": "rewardsClaimed",
      "discriminator": [
        75,
        98,
        88,
        18,
        219,
        112,
        88,
        121
      ]
    },
    {
      "name": "tokensStaked",
      "discriminator": [
        220,
        130,
        145,
        142,
        109,
        123,
        38,
        100
      ]
    },
    {
      "name": "tokensSwapped",
      "discriminator": [
        144,
        190,
        58,
        103,
        99,
        127,
        89,
        105
      ]
    },
    {
      "name": "tokensUnstaked",
      "discriminator": [
        137,
        203,
        131,
        80,
        135,
        107,
        181,
        150
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "unauthorized",
      "msg": "Unauthorized access"
    },
    {
      "code": 6001,
      "name": "adminAlreadyExists",
      "msg": "Admin already exists"
    },
    {
      "code": 6002,
      "name": "adminDoesntExist",
      "msg": "Admin doesn't exists"
    },
    {
      "code": 6003,
      "name": "maxAdminLimitReached",
      "msg": "Maximum admin limit reached"
    },
    {
      "code": 6004,
      "name": "invalidToken",
      "msg": "Invalid token input"
    },
    {
      "code": 6005,
      "name": "invalidTokenPair",
      "msg": "Invalid Token Pair"
    },
    {
      "code": 6006,
      "name": "mathOverflow",
      "msg": "Mathematical operation overflow"
    },
    {
      "code": 6007,
      "name": "invalidLiquidityAmount",
      "msg": "Invalid liquidity amount"
    },
    {
      "code": 6008,
      "name": "invalidSwapInput",
      "msg": "Invalid swap input"
    },
    {
      "code": 6009,
      "name": "excessiveSlippage",
      "msg": "Slippage tolerance exceeded"
    },
    {
      "code": 6010,
      "name": "insufficientFunds",
      "msg": "Insufficient funds"
    },
    {
      "code": 6011,
      "name": "invalidFeeRate",
      "msg": "Invalid fee rate"
    },
    {
      "code": 6012,
      "name": "invalidTokenRatio",
      "msg": "Invalid token ratio"
    },
    {
      "code": 6013,
      "name": "invalidStakeAmount",
      "msg": "Invalid stake amount"
    },
    {
      "code": 6014,
      "name": "insufficientStake",
      "msg": "Insufficient stake balance"
    },
    {
      "code": 6015,
      "name": "invalidLpTokenAmount",
      "msg": "Invalid LP token amount"
    },
    {
      "code": 6016,
      "name": "noLiquidity",
      "msg": "No liquidity provided"
    },
    {
      "code": 6017,
      "name": "noRewardsAvailable",
      "msg": "No rewards available to claim"
    },
    {
      "code": 6018,
      "name": "poolNotEmpty",
      "msg": "Pool is not empty"
    },
    {
      "code": 6019,
      "name": "invalidK",
      "msg": "Invalid K value after swap"
    }
  ],
  "types": [
    {
      "name": "adminUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "isAdded",
            "type": "bool"
          },
          {
            "name": "superAdmin",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "liquidityPool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "superAdmin",
            "type": "pubkey"
          },
          {
            "name": "admins",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "tokenXReserve",
            "type": "u64"
          },
          {
            "name": "tokenYReserve",
            "type": "u64"
          },
          {
            "name": "lpTokenSupply",
            "type": "u64"
          },
          {
            "name": "tokenXMint",
            "type": "pubkey"
          },
          {
            "name": "tokenYMint",
            "type": "pubkey"
          },
          {
            "name": "lpTokens",
            "type": {
              "vec": {
                "defined": {
                  "name": "lpTokenBalance"
                }
              }
            }
          },
          {
            "name": "feeRate",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "rewardRate",
            "type": "u64"
          },
          {
            "name": "totalStaked",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "liquidityProvided",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "tokenXAmount",
            "type": "u64"
          },
          {
            "name": "tokenYAmount",
            "type": "u64"
          },
          {
            "name": "lpTokensMinted",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "liquidityRemoved",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "tokenXAmount",
            "type": "u64"
          },
          {
            "name": "tokenYAmount",
            "type": "u64"
          },
          {
            "name": "lpTokensBurned",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "lpTokenBalance",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "lastRewardClaim",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "poolInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "feeRate",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "poolRemovedEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "rewardsClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "tokensStaked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "tokensSwapped",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "inputToken",
            "type": "string"
          },
          {
            "name": "inputAmount",
            "type": "u64"
          },
          {
            "name": "outputAmount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "tokensUnstaked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "rewards",
            "type": "u64"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "userState",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amountStaked",
            "type": "u64"
          },
          {
            "name": "lastStakeTimestamp",
            "type": "i64"
          },
          {
            "name": "rewardsEarned",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    }
  ]
};
