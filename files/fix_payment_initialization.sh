#!/bin/bash

. .env

sleep 15

yagna payment init --network ${PAYMENT_NETWORK} --driver ${PAYMENT_DRIVER} -g ${GSB_URL}