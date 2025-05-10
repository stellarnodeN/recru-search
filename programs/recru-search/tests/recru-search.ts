import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RecruSearch } from "../target/types/recru_search";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";

// Define custom error type
interface ProgramError {
  message: string;
  code?: number;
}

// Helper function to handle program errors
function isProgramError(error: unknown): error is ProgramError {
  return (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof (error as ProgramError).message === "string"
  );
}

describe("recru-search", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RecruSearch as Program<RecruSearch>;
  
  // Test accounts
  const admin = Keypair.generate();
  const researcher = Keypair.generate();
  const participant = Keypair.generate();
  const studyMint = Keypair.generate();
  const consentMint = Keypair.generate();
  
  // Helper function to create token account
  async function createTokenAccount(
    mint: PublicKey,
    owner: PublicKey
  ): Promise<PublicKey> {
    const tokenAccount = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: owner,
    });
    
    try {
      await program.methods
        .createTokenAccount()
        .accounts({
          tokenAccount,
          mint,
          owner,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();
        
      return tokenAccount;
    } catch (error) {
      if (isProgramError(error)) {
        throw new Error(`Failed to create token account: ${error.message}`);
      }
      throw error;
    }
  }

  // Helper function to initialize participant wallet
  async function initializeParticipantWallet(
    participant: PublicKey,
    phantomWallet: PublicKey
  ) {
    const [wallet] = await PublicKey.findProgramAddress(
      [Buffer.from("wallet"), participant.toBuffer()],
      program.programId
    );

    const mainTokenAccount = await createTokenAccount(studyMint.publicKey, participant);
    const privacyKeyAccount = await createTokenAccount(consentMint.publicKey, participant);

    try {
      await program.methods
        .initializePhantomWallet()
        .accounts({
          participant,
          wallet,
          mainTokenAccount,
          privacyKeyAccount,
          mint: studyMint.publicKey,
          phantomWallet,
          authority: participant,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (error) {
      if (isProgramError(error)) {
        throw new Error(`Failed to initialize participant wallet: ${error.message}`);
      }
      throw error;
    }
  }

  // Helper function to create a study
  async function createStudy(
    researcher: PublicKey,
    title: string,
    description: string,
    rewardAmount: number
  ) {
    const [study] = await PublicKey.findProgramAddress(
      [Buffer.from("study"), researcher.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .createStudy(
          title,
          description,
          "criteria_hash",
          new anchor.BN(rewardAmount),
          10 // max_participants
        )
        .accounts({
          study,
          researcher,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      return study;
    } catch (error) {
      if (isProgramError(error)) {
        throw new Error(`Failed to create study: ${error.message}`);
      }
      throw error;
    }
  }

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(researcher.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(participant.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
  });

  describe("Initialization", () => {
    it("Initializes the program", async () => {
      const [adminAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("admin")],
        program.programId
      );

      try {
        await program.methods
          .initialize()
          .accounts({
            admin: adminAccount,
            authority: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();

        const adminState = await program.account.admin.fetch(adminAccount);
        assert.ok(adminState.authority.equals(admin.publicKey));
      } catch (error) {
        if (isProgramError(error)) {
          throw new Error(`Failed to initialize program: ${error.message}`);
        }
        throw error;
      }
    });

    it("Initializes PsyPoints token", async () => {
      const [psypointsMint] = await PublicKey.findProgramAddress(
        [Buffer.from("psypoints")],
        program.programId
      );

      try {
        await program.methods
          .initializePsypoints()
          .accounts({
            mint: psypointsMint,
            authority: admin.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([admin])
          .rpc();

        const mintInfo = await provider.connection.getAccountInfo(psypointsMint);
        assert.ok(mintInfo !== null);
      } catch (error) {
        if (isProgramError(error)) {
          throw new Error(`Failed to initialize PsyPoints token: ${error.message}`);
        }
        throw error;
      }
    });
  });

  describe("Researcher Management", () => {
    it("Registers a new researcher", async () => {
      const [researcherAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("researcher"), researcher.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .registerResearcher(
            "Test University",
            "credentials_hash_123"
          )
          .accounts({
            researcher: researcherAccount,
            authority: researcher.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();

        const researcherState = await program.account.researcher.fetch(researcherAccount);
        assert.equal(researcherState.institution, "Test University");
        assert.equal(researcherState.credentialsHash, "credentials_hash_123");
        assert.equal(researcherState.isVerified, false);
        assert.equal(researcherState.studiesCreated, 0);
        assert.equal(researcherState.activeStudies, 0);
        assert.equal(researcherState.totalParticipants, 0);
        assert.equal(researcherState.reputationScore, 0);
      } catch (error) {
        if (isProgramError(error)) {
          throw new Error(`Failed to register researcher: ${error.message}`);
        }
        throw error;
      }
    });

    it("Verifies a researcher", async () => {
      const [researcherAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("researcher"), researcher.publicKey.toBuffer()],
        program.programId
      );

      const [adminAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("admin")],
        program.programId
      );

      await program.methods
        .verifyResearcher()
        .accounts({
          researcher: researcherAccount,
          admin: adminAccount,
          authority: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      const researcherState = await program.account.researcher.fetch(researcherAccount);
      assert.equal(researcherState.isVerified, true);
    });
  });

  describe("Study Management", () => {
    it("Creates a new study", async () => {
      const studyPubkey = await createStudy(
        researcher.publicKey,
        "Test Study",
        "This is a test study description",
        1000
      );

      const studyState = await program.account.study.fetch(studyPubkey);
      assert.equal(studyState.title, "Test Study");
      assert.equal(studyState.description, "This is a test study description");
      assert.equal(studyState.rewardAmount.toNumber(), 1000);
      assert.equal(studyState.maxParticipants, 10);
      assert.equal(studyState.currentParticipants, 0);
      assert.equal(studyState.isActive, true);
    });

    it("Updates study progress", async () => {
      const studyPubkey = await createStudy(
        researcher.publicKey,
        "Progress Test Study",
        "Testing study progress",
        1000
      );

      const [participantAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("participant"), participant.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .trackStudyProgress(50) // 50% progress
        .accounts({
          participant: participantAccount,
          study: studyPubkey,
          authority: participant.publicKey,
        })
        .signers([participant])
        .rpc();

      const studyState = await program.account.study.fetch(studyPubkey);
      assert.equal(studyState.progress, 50);
    });
  });

  describe("Participant Management", () => {
    it("Registers a new participant", async () => {
      const [participantAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("participant"), participant.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .registerParticipant("eligibility_proof_123")
        .accounts({
          participant: participantAccount,
          authority: participant.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([participant])
        .rpc();

      const participantState = await program.account.participant.fetch(participantAccount);
      assert.equal(participantState.eligibilityProof, "eligibility_proof_123");
      assert.equal(participantState.activeStudies, 0);
      assert.equal(participantState.completedStudies, 0);
    });

    it("Initializes participant wallet", async () => {
      const [participantAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("participant"), participant.publicKey.toBuffer()],
        program.programId
      );

      const phantomWallet = Keypair.generate();
      await initializeParticipantWallet(participantAccount, phantomWallet.publicKey);

      const [wallet] = await PublicKey.findProgramAddress(
        [Buffer.from("wallet"), participantAccount.toBuffer()],
        program.programId
      );

      const walletState = await program.account.participantWallet.fetch(wallet);
      assert.ok(walletState.phantomAddress.equals(phantomWallet.publicKey));
      assert.equal(walletState.isInitialized, true);
    });
  });

  describe("Consent Management", () => {
    it("Initializes consent NFT", async () => {
      const [consentMintAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("consent_mint")],
        program.programId
      );

      await program.methods
        .initializeConsentNft()
        .accounts({
          mint: consentMintAccount,
          authority: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc();

      const mintInfo = await provider.connection.getAccountInfo(consentMintAccount);
      assert.ok(mintInfo !== null);
    });

    it("Issues consent NFT to participant", async () => {
      const [participantAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("participant"), participant.publicKey.toBuffer()],
        program.programId
      );

      const [consentMintAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("consent_mint")],
        program.programId
      );

      const tokenAccount = await createTokenAccount(consentMintAccount, participant.publicKey);

      await program.methods
        .issueConsentNft("1.0", "consent_hash_123")
        .accounts({
          participant: participantAccount,
          mint: consentMintAccount,
          tokenAccount,
          authority: participant.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([participant])
        .rpc();

      const participantState = await program.account.participant.fetch(participantAccount);
      assert.equal(participantState.hasActiveConsent, true);
    });
  });

  describe("Reward System", () => {
    it("Completes study and distributes reward", async () => {
      const studyPubkey = await createStudy(
        researcher.publicKey,
        "Reward Test Study",
        "Testing reward distribution",
        1000
      );

      const [participantAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("participant"), participant.publicKey.toBuffer()],
        program.programId
      );

      const researcherTokenAccount = await createTokenAccount(studyMint.publicKey, researcher.publicKey);
      const participantTokenAccount = await createTokenAccount(studyMint.publicKey, participant.publicKey);

      await program.methods
        .completeStudy()
        .accounts({
          study: studyPubkey,
          participant: participantAccount,
          researcher: researcher.publicKey,
          researcherTokenAccount,
          participantTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([researcher])
        .rpc();

      const participantState = await program.account.participant.fetch(participantAccount);
      assert.equal(participantState.completedStudies, 1);
      assert.equal(participantState.activeStudies, 0);

      const studyState = await program.account.study.fetch(studyPubkey);
      assert.equal(studyState.completedParticipants, 1);
    });
  });

  describe("Error Cases", () => {
    describe("Researcher Errors", () => {
      it("Fails to create study with unverified researcher", async () => {
        const [researcherAccount] = await PublicKey.findProgramAddress(
          [Buffer.from("researcher"), researcher.publicKey.toBuffer()],
          program.programId
        );

        try {
          await createStudy(
            researcher.publicKey,
            "Invalid Study",
            "This should fail",
            1000
          );
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Researcher not verified");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });

      it("Fails to register duplicate researcher", async () => {
        const [researcherAccount] = await PublicKey.findProgramAddress(
          [Buffer.from("researcher"), researcher.publicKey.toBuffer()],
          program.programId
        );

        try {
          await program.methods
            .registerResearcher(
              "Test University",
              "credentials_hash_123"
            )
            .accounts({
              researcher: researcherAccount,
              authority: researcher.publicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([researcher])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Researcher already registered");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });
    });

    describe("Study Errors", () => {
      it("Fails to create study with invalid parameters", async () => {
        try {
          await program.methods
            .createStudy(
              "", // Empty title
              "Description",
              "criteria_hash",
              new anchor.BN(0), // Zero reward
              0 // Zero participants
            )
            .accounts({
              study: Keypair.generate().publicKey,
              researcher: researcher.publicKey,
              systemProgram: SystemProgram.programId,
            })
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Invalid study parameters");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });

      it("Fails to update progress beyond 100%", async () => {
        const studyPubkey = await createStudy(
          researcher.publicKey,
          "Progress Test Study",
          "Testing invalid progress",
          1000
        );

        try {
          await program.methods
            .trackStudyProgress(150) // Invalid progress > 100%
            .accounts({
              study: studyPubkey,
              participant: participant.publicKey,
              authority: participant.publicKey,
            })
            .signers([participant])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Invalid progress");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });
    });

    describe("Participant Errors", () => {
      it("Fails to register participant with invalid eligibility proof", async () => {
        try {
          const [participantAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("participant"), participant.publicKey.toBuffer()],
            program.programId
          );

          await program.methods
            .registerParticipant("") // Empty eligibility proof
            .accounts({
              participant: participantAccount,
              authority: participant.publicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([participant])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Invalid eligibility proof");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });

      it("Fails to initialize wallet with invalid Phantom address", async () => {
        try {
          const [participantAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("participant"), participant.publicKey.toBuffer()],
            program.programId
          );

          await initializeParticipantWallet(
            participantAccount,
            SystemProgram.programId // Invalid Phantom address
          );
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Invalid Phantom wallet");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });
    });

    describe("Consent Errors", () => {
      it("Fails to issue consent NFT without proper authorization", async () => {
        try {
          const [participantAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("participant"), participant.publicKey.toBuffer()],
            program.programId
          );

          const [consentMintAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("consent_mint")],
            program.programId
          );

          const tokenAccount = await createTokenAccount(consentMintAccount, participant.publicKey);

          await program.methods
            .issueConsentNft("1.0", "consent_hash_123")
            .accounts({
              participant: participantAccount,
              mint: consentMintAccount,
              tokenAccount,
              authority: researcher.publicKey, // Wrong authority
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([researcher])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Unauthorized consent issuance");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });

      it("Fails to revoke non-existent consent", async () => {
        try {
          const [participantAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("participant"), participant.publicKey.toBuffer()],
            program.programId
          );

          await program.methods
            .revokeConsent()
            .accounts({
              participant: participantAccount,
              authority: participant.publicKey,
            })
            .signers([participant])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "No active consent found");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });
    });

    describe("Reward System Errors", () => {
      it("Fails to complete study with insufficient funds", async () => {
        const studyPubkey = await createStudy(
          researcher.publicKey,
          "Reward Test Study",
          "Testing insufficient funds",
          1000000 // Large reward amount
        );

        try {
          const [participantAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("participant"), participant.publicKey.toBuffer()],
            program.programId
          );

          const researcherTokenAccount = await createTokenAccount(studyMint.publicKey, researcher.publicKey);
          const participantTokenAccount = await createTokenAccount(studyMint.publicKey, participant.publicKey);

          await program.methods
            .completeStudy()
            .accounts({
              study: studyPubkey,
              participant: participantAccount,
              researcher: researcher.publicKey,
              researcherTokenAccount,
              participantTokenAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([researcher])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Insufficient funds");
          } else {
            assert.fail("Unexpected error type");
          }
        });
      });

      it("Fails to complete study without proper progress", async () => {
        const studyPubkey = await createStudy(
          researcher.publicKey,
          "Progress Test Study",
          "Testing incomplete progress",
          1000
        );

        try {
          const [participantAccount] = await PublicKey.findProgramAddress(
            [Buffer.from("participant"), participant.publicKey.toBuffer()],
            program.programId
          );

          await program.methods
            .completeStudy()
            .accounts({
              study: studyPubkey,
              participant: participantAccount,
              researcher: researcher.publicKey,
              authority: participant.publicKey,
            })
            .signers([participant])
            .rpc();
          assert.fail("Should have thrown an error");
        } catch (error) {
          if (isProgramError(error)) {
            assert.include(error.message, "Study not completed");
          } else {
            assert.fail("Unexpected error type");
          }
        }
      });
    });
  });
});