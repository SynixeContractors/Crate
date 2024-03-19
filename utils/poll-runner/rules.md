Rules:
1. People can't vote twice
2. We want to know who voted
3. At no point can we know who voted for what

Who Voted:
- Prince
- John

Votes:
- Option B
- Option A

Prince voted for A
John voted for B

Create a poll.
All 5 staff members have to submit a public key.
Once all 5 keys are submitted, we generate a private key and public key, and shamir shard the private key
Store the public key and the shards
For each person that is allowed to vote, we generate a ticket
ticket {
    discord id
}
encrypt the ticket with public key
send everyone their ticket, we don't store them

when they vote
- ticket
- vote -> encrypted with the public and a salt

when we want to count the votes
we get 3 of the 5 keys
we combine them to get the private key
we decrypt the votes
we decrypt the tickets

List:
- Prince
- John

Votes:
- Option B
- Option A
