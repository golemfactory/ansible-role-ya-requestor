#!/bin/bash

. .env

yagna payment init --network ${PAYMENT_NETWORK} --driver ${PAYMENT_DRIVER} -g ${GSB_URL}