#!/bin/bash
set -e

function setup_tc {
    sudo tc qdisc add dev lo root handle 1: prio
    sudo tc qdisc add dev lo parent 1:1 handle 11: netem
    sudo tc filter add dev lo parent 1: prio 1 u32 \
        match ip protocol 17 0xff \
        match ip dport 12365 0xffff \
        flowid 1:1
    sudo tc filter add dev lo parent 1: prio 1 u32 \
        match ip protocol 17 0xff \
        match ip dport 12366 0xffff \
        flowid 1:1
}

function set_netem {
    sudo tc qdisc change dev lo parent 1:1 handle 11: netem $@
}

if [ -z "$(tc qdisc show dev lo | grep netem)" ]
then
    setup_tc
fi

set_netem $@

tc qdisc show dev lo | grep netem
