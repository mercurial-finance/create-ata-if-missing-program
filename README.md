# Create ATA if missing
In some situations we want to batch sending tokens, however we don't want to check and create ATA off-chain, as it is subject to race conditions (ATA created then second transaction attempts to create it again).

Instead we use this lightweight program, which will create the ATA if it does not exist.