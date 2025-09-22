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
    arrayTypeNode,
    fixedCountNode
} from 'codama';

export const root = rootNode(
    programNode({
        name: 'x_token',
        publicKey: '9Tqo4t4QYLxNe5HVxWo7zaav13j4pETEtkjyKf7a2VfG',
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
                        name: 'owner',
                        type: arrayTypeNode(numberTypeNode('u8'), fixedCountNode(32)),
                        docs: ['Owner username (max 32 bytes) - includes length in first byte'],
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
                    instructionArgumentNode({
                        name: 'initialBuyAmount',
                        type: numberTypeNode('u64'),
                        docs: ['Initial pre-buy token amount in base units (optional, 0 to skip)'],
                    }),
                    instructionArgumentNode({
                        name: 'initialMaxSol',
                        type: numberTypeNode('u64'),
                        docs: ['Max SOL (lamports) willing to pay for initial pre-buy'],
                    }),
                    instructionArgumentNode({
                        name: 'tokenName',
                        type: arrayTypeNode(numberTypeNode('u8'), fixedCountNode(32)),
                        docs: ['Token name (max 32 bytes) - includes length in first byte'],
                    }),
                    instructionArgumentNode({
                        name: 'tokenSymbol',
                        type: arrayTypeNode(numberTypeNode('u8'), fixedCountNode(10)),
                        docs: ['Token symbol (max 10 bytes) - includes length in first byte'],
                    }),
                    instructionArgumentNode({
                        name: 'tokenUri',
                        type: arrayTypeNode(numberTypeNode('u8'), fixedCountNode(200)),
                        docs: ['Token metadata URI (max 200 bytes) - includes length in first byte'],
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
                        name: 'treasury',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Treasury account (holds SOL for bonding curve)'],
                    }),
                    instructionAccountNode({
                        name: 'authorityTokenAccount',
                        isSigner: false,
                        isWritable: true,
                        docs: ["Authority's token account (ATA) to receive initial pre-buy tokens"],
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
                        name: 'associatedTokenProgram',
                        defaultValue: publicKeyValueNode(
                            'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
                            'associatedTokenProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['Associated Token Program'],
                    }),
                    instructionAccountNode({
                        name: 'rent',
                        isSigner: false,
                        isWritable: false,
                        docs: ['Rent sysvar'],
                    }),
                    instructionAccountNode({
                        name: 'feeRecipientAccount',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Fee recipient account (for initial pre-buy fee transfer)'],
                    }),
                    instructionAccountNode({
                        name: 'metadataAccount',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Metadata account (PDA from Metaplex)'],
                    }),
                    instructionAccountNode({
                        name: 'metaplexProgram',
                        defaultValue: publicKeyValueNode(
                            'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
                            'metaplexProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['Metaplex Token Metadata Program'],
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
                        name: 'treasury',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Treasury account (holds SOL for bonding curve)'],
                    }),
                    instructionAccountNode({
                        name: 'feeRecipient',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Fee recipient account'],
                    }),
                    instructionAccountNode({
                        name: 'tradingStats',
                        isSigner: false,
                        isWritable: true,
                        docs: ["Buyer's trading stats account"],
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
                        name: 'treasury',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Treasury account (holds SOL for bonding curve)'],
                    }),
                    instructionAccountNode({
                        name: 'feeRecipient',
                        isSigner: false,
                        isWritable: true,
                        docs: ['Fee recipient account'],
                    }),
                    instructionAccountNode({
                        name: 'tradingStats',
                        isSigner: false,
                        isWritable: true,
                        docs: ["Seller's trading stats account"],
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
                        name: 'systemProgram',
                        defaultValue: publicKeyValueNode(
                            '11111111111111111111111111111111',
                            'systemProgram'
                        ),
                        isSigner: false,
                        isWritable: false,
                        docs: ['System Program'],
                    }),
                ],
            }),
            instructionNode({
                name: 'updateProfile',
                discriminators: [
                    constantDiscriminatorNode(
                        constantValueNode(numberTypeNode('u8'), numberValueNode(3))
                    ),
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(3),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({
                        name: 'usernameLen',
                        type: numberTypeNode('u8'),
                        docs: ['Username length (max 32 characters)'],
                    }),
                    instructionArgumentNode({
                        name: 'bioLen',
                        type: numberTypeNode('u8'),
                        docs: ['Bio length (max 200 characters)'],
                    }),
                    instructionArgumentNode({
                        name: '_padding',
                        type: numberTypeNode('u16'),
                        docs: ['Padding for alignment'],
                    }),
                    instructionArgumentNode({
                        name: 'username',
                        type: arrayTypeNode(numberTypeNode('u8'), fixedCountNode(32)),
                        docs: ['Username (32 bytes)'],
                    }),
                    instructionArgumentNode({
                        name: 'bio',
                        type: arrayTypeNode(numberTypeNode('u8'), fixedCountNode(200)),
                        docs: ['Bio (200 bytes)'],
                    }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: 'userProfile',
                        isSigner: false,
                        isWritable: true,
                        docs: ['User profile account (PDA)'],
                    }),
                    instructionAccountNode({
                        name: 'user',
                        isSigner: true,
                        isWritable: true,
                        docs: ['User wallet (must be signer)'],
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
                ],
            }),
            instructionNode({
                name: 'getLeaderboard',
                discriminators: [
                    constantDiscriminatorNode(
                        constantValueNode(numberTypeNode('u8'), numberValueNode(4))
                    ),
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(4),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({
                        name: 'limit',
                        type: numberTypeNode('u8'),
                        docs: ['Number of top traders to return (max 100)'],
                    }),
                    instructionArgumentNode({
                        name: 'offset',
                        type: numberTypeNode('u8'),
                        docs: ['Offset for pagination'],
                    }),
                ],
                accounts: [
                    // No accounts needed for this instruction as it's read-only
                    // The data will be returned via program logs or client-side account scanning
                ],
            }),
        ],
    })
);