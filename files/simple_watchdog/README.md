# Simple watchdog

Checks if yagna works correctly. If it's not, it resolves problems by wiping datadir. Suitable only for testnet payments.


## Monitored anomalies

- log "insufficient funds for gas * price + value"
