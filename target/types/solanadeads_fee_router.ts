/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/solanadeads_fee_router.json`.
 */
export type SolanadeadsFeeRouter = {
  "address": "DEADS3ucNHjN8iz3Cw65joYxgVdguNsjytHRqCs7QvzA",
  "metadata": {
    "name": "solanadeadsFeeRouter",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Solana Deads Fee Router (Token-2022 compatible)"
  },
  "instructions": [
    {
      "name": "distributeFees",
      "docs": [
        "Distribute a specific amount from the router vault."
      ],
      "discriminator": [
        120,
        56,
        27,
        7,
        53,
        176,
        113,
        186
      ],
      "accounts": [
        {
          "name": "router",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  111,
                  108,
                  97,
                  110,
                  97,
                  100,
                  101,
                  97,
                  100,
                  115
                ]
              },
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101,
                  45,
                  114,
                  111,
                  117,
                  116,
                  101,
                  114,
                  45,
                  118,
                  49
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "mint"
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "routerVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "router"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "treasuryOwner"
        },
        {
          "name": "treasuryWallet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "treasuryOwner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "lpOwner"
        },
        {
          "name": "lpPoolWallet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "lpOwner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "stakersOwner"
        },
        {
          "name": "stakersWallet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "stakersOwner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "decimals",
          "type": "u8"
        }
      ]
    },
    {
      "name": "harvestAndDistribute",
      "docs": [
        "Harvest withheld fees, withdraw to vault, then distribute.",
        "`remaining_accounts` should be the list of **fee-bearing token accounts** to harvest from."
      ],
      "discriminator": [
        161,
        109,
        210,
        126,
        12,
        152,
        40,
        15
      ],
      "accounts": [
        {
          "name": "router",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  111,
                  108,
                  97,
                  110,
                  97,
                  100,
                  101,
                  97,
                  100,
                  115
                ]
              },
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101,
                  45,
                  114,
                  111,
                  117,
                  116,
                  101,
                  114,
                  45,
                  118,
                  49
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "mint",
          "writable": true
        },
        {
          "name": "tokenProgram"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "routerVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "router"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "treasuryOwner"
        },
        {
          "name": "treasuryWallet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "treasuryOwner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "lpOwner"
        },
        {
          "name": "lpPoolWallet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "lpOwner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "stakersOwner"
        },
        {
          "name": "stakersWallet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "stakersOwner"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        }
      ],
      "args": []
    },
    {
      "name": "initializeRouter",
      "discriminator": [
        115,
        12,
        152,
        202,
        152,
        4,
        87,
        120
      ],
      "accounts": [
        {
          "name": "router",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  111,
                  108,
                  97,
                  110,
                  97,
                  100,
                  101,
                  97,
                  100,
                  115
                ]
              },
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101,
                  45,
                  114,
                  111,
                  117,
                  116,
                  101,
                  114,
                  45,
                  118,
                  49
                ]
              },
              {
                "kind": "account",
                "path": "mint"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "mint"
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "router",
      "discriminator": [
        94,
        226,
        217,
        169,
        186,
        4,
        198,
        7
      ]
    }
  ],
  "events": [
    {
      "name": "feeDistribution",
      "discriminator": [
        84,
        6,
        82,
        237,
        116,
        16,
        120,
        34
      ]
    },
    {
      "name": "harvestRun",
      "discriminator": [
        74,
        176,
        100,
        183,
        123,
        52,
        163,
        28
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "zeroAmount",
      "msg": "Input amount must be greater than or equal to the minimum threshold"
    },
    {
      "code": 6001,
      "name": "mathOverflow",
      "msg": "Math overflow while computing splits"
    },
    {
      "code": 6002,
      "name": "insufficientVaultBalance",
      "msg": "Router vault has insufficient balance for requested distribution"
    },
    {
      "code": 6003,
      "name": "decimalsMismatch",
      "msg": "Provided decimals do not match the mint's decimals"
    }
  ],
  "types": [
    {
      "name": "feeDistribution",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakersAmount",
            "type": "u64"
          },
          {
            "name": "treasuryAmount",
            "type": "u64"
          },
          {
            "name": "lpAmount",
            "type": "u64"
          },
          {
            "name": "total",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "harvestRun",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sources",
            "type": "u32"
          },
          {
            "name": "vaultBefore",
            "type": "u64"
          },
          {
            "name": "distributed",
            "type": "u64"
          },
          {
            "name": "vaultAfter",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "router",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "authority",
            "type": "pubkey"
          }
        ]
      }
    }
  ]
};
