import {
    rootNode,
    programNode,
    instructionNode,
    constantDiscriminatorNode,
    constantValueNode,
    numberTypeNode,
    numberValueNode,
    instructionArgumentNode,
    instructionAccountNode,
    publicKeyValueNode,
    publicKeyTypeNode,
} from 'codama';

export const root = rootNode(
    programNode({
        name: 'x_token',
        publicKey: '8ngixonBmuMvzKA3woFu9rgjXqTtZYM4vMsmNgf9KF7S',
        version: '1.0.0',
        instructions: [
            instructionNode({
                name: 'initialize',
                discriminators: [
                    constantDiscriminatorNode(
                        constantValueNode(numberTypeNode('u8'), numberValueNode(0))
                    ),
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(0),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({
                        name: 'decimals',
                        type: numberTypeNode('u8'),
                        docs: ['The number of decimals for the token.'],
                    }),
                    instructionArgumentNode({
                        name: 'curveType',
                        type: numberTypeNode('u8'),
                        docs: ['Curve type (0 = linear, 1 = exponential, 2 = logarithmic)'],
                    }),
                    instructionArgumentNode({
                        name: 'feeBasisPoints',
                        type: numberTypeNode('u16'),
                        docs: ['Fees in basis points (100 = 1%)'],
                    }),
                    instructionArgumentNode({
                        name: '_padding',
                        type: numberTypeNode('u32'),
                        docs: ['Padding for alignment'],
                    }),
                    instructionArgumentNode({
                        name: 'basePrice',
                        type: numberTypeNode('u64'),
                        docs: ['Base price in lamports per token (scaled by 1e9)'],
                    }),
                    instructionArgumentNode({
                        name: 'slope',
                        type: numberTypeNode('u64'),
                        docs: ['Slope parameter for pricing curve (scaled by 1e9)'],
                    }),
                    instructionArgumentNode({
                        name: 'maxSupply',
                        type: numberTypeNode('u64'),
                        docs: ['Maximum token supply'],
                    }),
                    instructionArgumentNode({
                        name: 'feeRecipient',
                        type: publicKeyTypeNode(),
                        docs: ['Fee recipient address'],
                    }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: 'authority',
                        isSigner: true,
                        isWritable: false,
                        docs: ['Authority that will control the bonding curve'],
                    }),
                    instructionAccountNode({
                        name: 'bondingCurve',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Bonding curve state account (PDA) - will be created by program'],
                    }),
                    instructionAccountNode({
                        name: 'mint',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Token mint account - must be created by client before calling'],
                    }),
                    instructionAccountNode({
                        name: 'payer',
                        isSigner: true,
                        isWritable: true,
                        docs: ['Payer for account creation and rent'],
                    }),
                    instructionAccountNode({
                        name: 'systemProgram',
                        defaultValue: publicKeyValueNode(
                            '11111111111111111111111111111111',
                            'systemProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['System Program'],
                    }),
                    instructionAccountNode({
                        name: 'tokenProgram',
                        defaultValue: publicKeyValueNode(
                            'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
                            'tokenProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['Token Program'],
                    }),
                    instructionAccountNode({
                        name: 'rent',
                        isSigner: false,
                        isWritable: false,
                        docs: ['Rent sysvar'],
                    }),
                ],
            }),
            instructionNode({
                name: 'buyTokens',
                discriminators: [
                    constantDiscriminatorNode(
                        constantValueNode(numberTypeNode('u8'), numberValueNode(1))
                    ),
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(1),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({
                        name: 'tokenAmount',
                        type: numberTypeNode('u64'),
                        docs: ['Amount of tokens to buy'],
                    }),
                    instructionArgumentNode({
                        name: 'maxSolAmount',
                        type: numberTypeNode('u64'),
                        docs: ['Maximum SOL amount willing to pay (slippage protection)'],
                    }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: 'buyer',
                        isSigner: true,
                        isWritable: true,
                        docs: ['Buyer account'],
                    }),
                    instructionAccountNode({
                        name: 'bondingCurve',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Bonding curve state account'],
                    }),
                    instructionAccountNode({
                        name: 'mint',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Token mint account'],
                    }),
                    instructionAccountNode({
                        name: 'buyerTokenAccount',
                        isSigner: false,
                        isWritable: true,
                        docs: ["Buyer's token account (will be created if doesn't exist)"],
                    }),
                    instructionAccountNode({
                        name: 'feeRecipient',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Fee recipient account'],
                    }),
                    instructionAccountNode({
                        name: 'systemProgram',
                        defaultValue: publicKeyValueNode(
                            '11111111111111111111111111111111',
                            'systemProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['System Program'],
                    }),
                    instructionAccountNode({
                        name: 'tokenProgram',
                        defaultValue: publicKeyValueNode(
                            'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
                            'tokenProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['Token Program'],
                    }),
                    instructionAccountNode({
                        name: 'associatedTokenProgram',
                        defaultValue: publicKeyValueNode(
                            'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
                            'associatedTokenProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['Associated Token Program'],
                    }),
                ],
            }),
            instructionNode({
                name: 'sellTokens',
                discriminators: [
                    constantDiscriminatorNode(
                        constantValueNode(numberTypeNode('u8'), numberValueNode(2))
                    ),
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(2),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({
                        name: 'tokenAmount',
                        type: numberTypeNode('u64'),
                        docs: ['Amount of tokens to sell'],
                    }),
                    instructionArgumentNode({
                        name: 'minSolAmount',
                        type: numberTypeNode('u64'),
                        docs: ['Minimum SOL amount willing to accept (slippage protection)'],
                    }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: 'seller',
                        isSigner: true,
                        isWritable: true,
                        docs: ['Seller account'],
                    }),
                    instructionAccountNode({
                        name: 'bondingCurve',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Bonding curve state account'],
                    }),
                    instructionAccountNode({
                        name: 'mint',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Token mint account'],
                    }),
                    instructionAccountNode({
                        name: 'sellerTokenAccount',
                        isSigner: false,
                        isWritable: true,
                        docs: ["Seller's token account"],
                    }),
                    instructionAccountNode({
                        name: 'feeRecipient',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Fee recipient account'],
                    }),
                    instructionAccountNode({
                        name: 'tokenProgram',
                        defaultValue: publicKeyValueNode(
                            'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
                            'tokenProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['Token Program'],
                    }),
                ],
            }),
        ],
    })
);