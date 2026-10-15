# Borrowed Badge

The prize contains 10 SOL. The server gives you a valid SPL Token account that
holds the only badge token, but that token account belongs to somebody else.
Drain the prize.

Review `challenge/src/lib.rs` and implement the exploit in `solve/src/lib.rs`.

## Build

```bash
docker build --platform linux/amd64 -t borrowed-badge-solve .
```

## Submit

```bash
docker run --rm --platform linux/amd64 borrowed-badge-solve HOST PORT
```

For the local organizer instance, use `host.docker.internal 5001` on Docker
Desktop.

The server executes one instruction against your uploaded `solve.so`. The
transaction payer is passed to your program as a signer.

## Program API

- `[0] RedeemReward`: validates a membership badge and transfers the reward
- `[1] VerifyMembership`: returns the badge authority and balance
- `[2] GetRewardVaultBalance`: returns the available reward balance
- `[3] GetCampaignTerms`: returns the required decimals and badge supply

## Account order

1. player, signer and writable
2. canonical badge mint, read-only
3. borrowed badge token account, read-only
4. prize PDA, writable
5. challenge program, executable

## Review hint

`AccountInfo.owner` tells you which program owns the serialized bytes. After
unpacking an SPL Token account, `TokenAccount.owner` is the token authority.
The two fields answer different questions.
