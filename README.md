# Taplocks - Verifiable but unspendable tapleafs (with an application to emulating new opcodes)

There might be other applications for this. We came up with it in the context of emulating future opcodes via a policy signer. 

Here's the motivation: suppose you wanted to emulate some not-yet-active future opcode (like OP_CAT or CTV or a new post-quantum signing scheme, or something else). You could have a trusted signer that will sign transactions according to the semantics of tapscript with those future opcodes. For example, you have a signer that enforces OP_CAT. It hands out a pubkey, and you can encumber funds with it. To spend the output, you send a PSBT to the signer along with some witness data. The signer executes your script with the witness data and if it's valid, signs the transaction. In this way you can emulate not-yet-active opcodes on mainnet. To reduce trust in a single signer, you set up a multisig with multiple policy signers. This lets you try out future opcodes now (if you have an application where a (federated?) policy signer is an acceptable trust assumption).

Now what would be *really* cool is if in addition to the emulated-checksig spend path, your UTXOs also had a teapleaf with the real actual script using those opcodes. So if you are building some application with CAT or CTV, you have a tapleaf that uses the policy-signers to just do CHECKSIG, and then you have another tapleaf with the actual CAT/CTV/CCV/whatever script. Then when the opcode(s) in question become active on the network, you have your emulation servers just delete their keys (if it's a federation, at least N-T+1 members must delete their keys then it's safe) and then users can spend their UTXOs along the real script paths. In this setup, imagine a taproot address that commits to two (or more) tapleafs:

- CHECKSIG with one or more public keys from emulation servers
- The actual script using the new opcodes


The problem here is that in Tapscript, we are going to add new opcodes with OP_SUCCESS. A script containing OP_SUCCESS succeeds, no matter what. Even if that OP_SUCCESS is behind `OP_FALSE OP_IF`, the script will still succeed. So You can't simply make a tapleaf with CAT (or whatever opcode), because anyone who can construct the tapleaf will be able to spend it until the network enforces the semantic of the new opcode.

So that motivates a construction called a Taplock: can we "lock" a tapleaf so that users cant spend it? If we could, we could have locked tapleafs containing unactivated opcodes that we unlock once the opcode is active. The tldr is that for the trust model above we can do this, (ab)using sha256 length extension and ZKPs. 

Here's how it works:

Spending a Taproot output on a script path involves verifying that the script was committed to in a tapleaf, and the executing the script. When verifying that the script is included in the taproot address, you need two things: the script itself and the control block (which is the last non-annex element on the stack) which contains a merkle authentication path that is used to verify inclusion of the script in the taptree (for the specific details, see BIP-341). When doing this validation, you have to first construct a tagged hash of the script, called the `tapleaf hash`. The tapleaf hash is constructed as `hashTapLeaf(v || compact_size(size of s) || s)` where `s` is the script and `v` is the leaf version. 

Now, notice that you have to *know* `s` in order to construct the leaf hash (or for the script interpreter to execute the script at all). You have a hash commitment in the taproot commitment, and you have to produce the preimage to it in order to spend it. In other words, the tapleaf itself is a kind of a hashlock! So a super trivial -- but bad -- way to have a "secret" tapleaf would be:

- some semi-trusted oracle constructs the tapleaf and the leafhash
- users get the leafhash from the oracle (in a public registry, baked into their wallet, through an RPC, whatever)
- users use the leafhash to build their taptree, but can't spend with it. 
- When the opcode gets activated, the oracle publishes the script, users can spend from it

This sort of works but is very problematic. For one thing, users might be able to guess the script construction, and then this leaf reduces to op_success. Secondly, users need to know the semantics of that tapleaf. Especially in multi-party constructions, committing to completely opaque tapleaf commitments is a recipe for disaster. So what we want instead is a way that users can know what is in a script, but still not be able to produce `s` to build a valid leafhash preimage in the transaction witness.

SHA-256 is a member of a family of hash functions that use a Merkle–Damgård construction, which is susceptible to length extension. SHA-256 processes data in 64 byte chunks that updates the internal state of the hash engine. That means that if you take chunk A, run it through the SHA-256 internal compression function, take the midstate, and then apply B to it, that is the equivalent of just hashing A||B. We can use this property to provide what is essentially a hashlock on a tapleaf. 

Alice has some script S that she wants to Taplock (she wants it to be in a tapleaf, but she doesn't want to be able to construct the tapleaf). She sends the length of the script S to Oscar the oracle. Oscar creates and securely saves a secret value L (for Lock). He also computes a pad_len which is 27 + num_bytes(compact_size(size of s)). Oscar then makes the following script fragment:

```
PUSHBYTES_32
L
OP_DROP
[padding]
```

where the padding consists of pad_len/2 repetitions of `OP_1 OP_DROP`. We'll call this script fragment the "header" Oscar the Oracle then computes the hash midstate of `hashTapLeaf(v || compact_size(size of s + header ) || header)`. Think of this as a checkpoint of if Oscar was building a leafhash of (header || S), but just stopped once he had hashed the header. We'll call this hash midstate the "Header hash" (note that the padding in the header is so that `v || compact_size(size of s + header ) || header` gets to 64 bytes, which is the chunk size for sha-256). They also construct a ZKP that the header hash is correctly constructed. Oscar then sends the header hash and the proof to Alice.

Alice's software then loads the Header hash (which is really a hash midstate) into a sha256 engine, and then continues hashing the rest of the script S, and completes the hash. She now has the leafhash of the script `header || S`, which is just 

```
PUSHBYTES_32
L
OP_DROP
[padding]
S...
```

She knows that the header doesn't contain malicious code and only contains the data pushes and some no-op opcodes because of the ZKP. But she cannot construct the whole script without knowledge of L. So, she can commit to a script where she knows the behavior, but she can not spend from that script until L is revealed by Oscar. The tapleaf is locked until L is published.

Now Alice might not want to rely on only one Oracle. She can rely on multiple oracles by stacking these headers. The first oracle computes a secret value and a header, constructs the midstate and sends it and a proof to the second oracle, who repeats the process. By the end, Alice gets a hash midstate and a list of proofs that commit to N secret values. All the secret values must be revealed in order to construct the tapleaf. This N-of-N trust assumption can be made into a threshold scheme (t-of-N) by having multiple tapleafs with different quorums of oracles. 

Whether using a single oracle or multiple oracles, this is a mechanism where an OP_SUCCESS opcode can be included in a tapleaf that is spendable only after an oracle publishes a secret value. 

Other possible enhancements: an oracle can also post a bond that is encumbered by a Taplocked leaf, such that if they reveal L early, they lose their bond. Or users of the Oracle could create a payment to the oracle that is both hashlocked by H(L) and timelocked to a blockheight after the expected activation of whatever softfork, to pay the oracle to reveal L after the timelock expires.

Thanks to James O'Beirne, Jeremy Ruben, and Jesse Posner for feedback. 