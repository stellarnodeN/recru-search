import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount, closeAccount } from "@solana/spl-token";
import { expect } from "chai";

// Import the IDL type
import { RecruSearch } from "../target/types/recru_search";

// Define types until the generated types are available
type StudyType = {
  survey: Record<string, never>;
} | {
  interview: Record<string, never>;
} | {
  clinical: Record<string, never>;
} | {
  observational: Record<string, never>;
} | {
  experimental: Record<string, never>;
};

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
  const psyPointsMint = Keypair.generate();
  
  // Token accounts
  let researcherTokenAccount: PublicKey;
  let participantTokenAccount: PublicKey;
  let psyPointsTokenAccount: PublicKey;
  
  // PDAs
  const [adminPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("admin")],
    program.programId
  );
  
  const [researcherPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("researcher"), researcher.publicKey.toBuffer()],
    program.programId
  );
  
  const [participantPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("participant"), participant.publicKey.toBuffer()],
    program.programId
  );

  let studyPda: PublicKey;
  let isInitialized = false;

  // Setup before all tests
  before(async () => {
    try {
      // Airdrop SOL to test accounts
      const airdropAmount = 2 * LAMPORTS_PER_SOL;
      await provider.connection.requestAirdrop(admin.publicKey, airdropAmount);
      await provider.connection.requestAirdrop(researcher.publicKey, airdropAmount);
      await provider.connection.requestAirdrop(participant.publicKey, airdropAmount);
      
      // Create token mints
      await createMint(
        provider.connection,
        admin,
        admin.publicKey,
        null,
        9,
        studyMint
      );
      
      await createMint(
        provider.connection,
        admin,
        admin.publicKey,
        null,
        9,
        psyPointsMint
      );

      // Create token accounts
      researcherTokenAccount = await createAccount(
        provider.connection,
        researcher,
        studyMint.publicKey,
        researcher.publicKey
      );

      participantTokenAccount = await createAccount(
        provider.connection,
        participant,
        studyMint.publicKey,
        participant.publicKey
      );

      psyPointsTokenAccount = await createAccount(
        provider.connection,
        participant,
        psyPointsMint.publicKey,
        participant.publicKey
      );

      // Mint tokens to researcher for study rewards
      await mintTo(
        provider.connection,
        admin,
        studyMint.publicKey,
        researcherTokenAccount,
        admin.publicKey,
        10000000000 // 10 tokens
      );

      // Initialize study PDA
      [studyPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcherPda.toBuffer()],
        program.programId
      );
    } catch (error) {
      console.error("Setup error:", error);
      throw error;
    }
  });

  // Cleanup after all tests
  after(async () => {
    try {
      // Close token accounts if they exist
      const accounts = [researcherTokenAccount, participantTokenAccount, psyPointsTokenAccount];
      for (const account of accounts) {
        try {
          const accountInfo = await getAccount(provider.connection, account);
          if (accountInfo) {
            await closeAccount(
              provider.connection,
              admin,
              account,
              admin.publicKey,
              admin
            );
          }
        } catch (error) {
          console.log(`Account ${account.toString()} already closed or not found`);
        }
      }
    } catch (error) {
      console.error("Cleanup error:", error);
    }
  });

  describe("Initialization", () => {
    it("Initializes the program", async () => {
      try {
        const tx = await program.methods
          .initialize()
          .accounts({
            admin: adminPda,
            authority: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
        
        console.log("Program initialization tx:", tx);
        
        // Verify admin account was created
        const adminAccount = await program.account.admin.fetch(adminPda);
        expect(adminAccount.authority).to.eql(admin.publicKey);
        isInitialized = true;
      } catch (error) {
        console.error("Initialization error:", error);
        throw error;
      }
    });

    it("Fails to initialize program twice", async () => {
      if (!isInitialized) {
        console.log("Skipping test - program not initialized");
        return;
      }

      try {
        await program.methods
          .initialize()
          .accounts({
            admin: adminPda,
            authority: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Initializes PsyPoints token", async () => {
      try {
        const tx = await program.methods
          .initializePsyPoints()
          .accounts({
            mint: psyPointsMint.publicKey,
            authority: admin.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([admin])
          .rpc();
        
        console.log("PsyPoints initialization tx:", tx);
      } catch (error) {
        console.error("PsyPoints initialization error:", error);
        throw error;
      }
    });
  });

  describe("Researcher Management", () => {
    it("Registers a new researcher", async () => {
      try {
        const institution = "Test University";
        const credentialsHash = "test_credentials_hash";
        
        const tx = await program.methods
          .registerResearcher(institution, credentialsHash)
          .accounts({
            researcher: researcherPda,
            authority: researcher.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        console.log("Researcher registration tx:", tx);
        
        // Verify researcher account
        const researcherAccount = await program.account.researcher.fetch(researcherPda);
        expect(researcherAccount.institution).to.equal(institution);
        expect(researcherAccount.credentialsHash).to.equal(credentialsHash);
        expect(researcherAccount.isVerified).to.be.false;
      } catch (error) {
        console.error("Researcher registration error:", error);
        throw error;
      }
    });

    it("Fails to register same researcher twice", async () => {
      try {
        const institution = "Test University";
        const credentialsHash = "test_credentials_hash";
        
        await program.methods
          .registerResearcher(institution, credentialsHash)
          .accounts({
            researcher: researcherPda,
            authority: researcher.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Verifies a researcher", async () => {
      try {
        const tx = await program.methods
          .verifyResearcher()
          .accounts({
            researcher: researcherPda,
            admin: adminPda,
            authority: admin.publicKey,
          })
          .signers([admin])
          .rpc();
        
        console.log("Researcher verification tx:", tx);
        
        // Verify researcher is now verified
        const researcherAccount = await program.account.researcher.fetch(researcherPda);
        expect(researcherAccount.isVerified).to.be.true;
      } catch (error) {
        console.error("Researcher verification error:", error);
        throw error;
      }
    });

    it("Fails to verify researcher with non-admin", async () => {
      try {
        await program.methods
          .verifyResearcher()
          .accounts({
            researcher: researcherPda,
            admin: adminPda,
            authority: researcher.publicKey,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Study Management", () => {
    it("Creates a new study", async () => {
      try {
        const title = "Test Study";
        const description = "A test study for research";
        const criteriaHash = "criteria123";
        const rewardAmount = new anchor.BN(1000000000); // 1 token
        const maxParticipants = 10;

        const tx = await program.methods
          .createStudy(
            title,
            description,
            criteriaHash,
            rewardAmount,
            maxParticipants
          )
          .accounts({
            study: studyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();

        console.log("Study creation tx:", tx);

        // Verify study was created
        const studyAccount = await program.account.study.fetch(studyPda);
        expect(studyAccount.title).to.equal(title);
        expect(studyAccount.description).to.equal(description);
        expect(studyAccount.criteriaHash).to.equal(criteriaHash);
        expect(studyAccount.rewardAmount.toString()).to.equal(rewardAmount.toString());
        expect(studyAccount.maxParticipants).to.equal(maxParticipants);
        expect(studyAccount.currentParticipants).to.equal(0);
        expect(studyAccount.isActive).to.be.true;
      } catch (error) {
        console.error("Study creation error:", error);
        throw error;
      }
    });

    it("Fails to create study with unverified researcher", async () => {
      const unverifiedResearcher = Keypair.generate();
      const [unverifiedResearcherPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("researcher"), unverifiedResearcher.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .createStudy(
            "Test Study",
            "A test study for research",
            "test_criteria_hash",
            new anchor.BN(1000000000),
            10
          )
          .accounts({
            study: studyPda,
            researcher: unverifiedResearcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([unverifiedResearcher])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Fails to create study with invalid parameters", async () => {
      try {
        await program.methods
          .createStudy(
            "Test Study",
            "Test Description",
            "test_criteria_hash",
            0, // Invalid reward amount
            10
          )
          .accounts({
            study: studyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid reward amount");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Fails to create study with excessive max participants", async () => {
      try {
        await program.methods
          .createStudy(
            "Test Study",
            "Test Description",
            "test_criteria_hash",
            1000,
            2000 // Exceeds maximum limit
          )
          .accounts({
            study: studyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error for excessive max participants");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Participant Management", () => {
    it("Registers a new participant", async () => {
      try {
        const eligibilityProof = "proof123";

        const tx = await program.methods
          .registerParticipant(eligibilityProof)
          .accounts({
            participant: participantPda,
            authority: participant.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([participant])
          .rpc();

        console.log("Participant registration tx:", tx);

        // Verify participant was registered
        const participantAccount = await program.account.participant.fetch(participantPda);
        expect(participantAccount.authority).to.eql(participant.publicKey);
        expect(participantAccount.eligibilityProof).to.equal(eligibilityProof);
        expect(participantAccount.activeStudies).to.equal(0);
        expect(participantAccount.completedStudies).to.equal(0);
      } catch (error) {
        console.error("Participant registration error:", error);
        throw error;
      }
    });

    it("Fails to register same participant twice", async () => {
      try {
        await program.methods
          .registerParticipant("test_eligibility_proof")
          .accounts({
            participant: participantPda,
            authority: participant.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([participant])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Initializes Phantom wallet for participant", async () => {
      try {
        const tx = await program.methods
          .initializePhantomWallet()
          .accounts({
            participant: participantPda,
            authority: participant.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([participant])
          .rpc();
        
        console.log("Phantom wallet initialization tx:", tx);
        
        // Verify participant wallet was initialized
        const participantAccount = await program.account.participant.fetch(participantPda);
        expect(participantAccount.wallet).to.exist;
      } catch (error) {
        console.error("Phantom wallet initialization error:", error);
        throw error;
      }
    });
  });

  describe("Study Participation", () => {
    it("Allows participant to join a study", async () => {
      try {
        const tx = await program.methods
          .joinStudy()
          .accounts({
            study: studyPda,
            participant: participantPda,
            authority: participant.publicKey,
          })
          .signers([participant])
          .rpc();
        
        console.log("Join study tx:", tx);
        
        // Verify study and participant accounts were updated
        const studyAccount = await program.account.study.fetch(studyPda);
        const participantAccount = await program.account.participant.fetch(participantPda);
        
        expect(studyAccount.currentParticipants).to.equal(1);
        expect(participantAccount.activeStudies).to.equal(1);
      } catch (error) {
        console.error("Join study error:", error);
        throw error;
      }
    });

    it("Fails to join study when at capacity", async () => {
      // Create a study at capacity
      const fullStudyMint = Keypair.generate();
      const [fullStudyPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("study"), researcherPda.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .createStudy(
            "Full Study",
            "A study at capacity",
            "test_criteria_hash",
            new anchor.BN(1000000000),
            1 // maxParticipants = 1
          )
          .accounts({
            study: fullStudyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();

        // Try to join with a second participant
        const secondParticipant = Keypair.generate();
        const [secondParticipantPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("participant"), secondParticipant.publicKey.toBuffer()],
          program.programId
        );

        await program.methods
          .joinStudy()
          .accounts({
            study: fullStudyPda,
            participant: secondParticipantPda,
            authority: secondParticipant.publicKey,
          })
          .signers([secondParticipant])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Tracks study progress", async () => {
      try {
        const progress = 50; // 50% complete
        
        const tx = await program.methods
          .trackStudyProgress(progress)
          .accounts({
            study: studyPda,
            participant: participantPda,
            authority: participant.publicKey,
          })
          .signers([participant])
          .rpc();
        
        console.log("Study progress tx:", tx);
      } catch (error) {
        console.error("Study progress error:", error);
        throw error;
      }
    });

    it("Fails to track progress for non-participant", async () => {
      const nonParticipant = Keypair.generate();
      const [nonParticipantPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("participant"), nonParticipant.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .trackStudyProgress(50)
          .accounts({
            study: studyPda,
            participant: nonParticipantPda,
            authority: nonParticipant.publicKey,
          })
          .signers([nonParticipant])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Submits study feedback", async () => {
      try {
        const rating = 5;
        const feedback = "Great study experience!";
        
        const tx = await program.methods
          .submitStudyFeedback(rating, feedback)
          .accounts({
            study: studyPda,
            participant: participantPda,
            authority: participant.publicKey,
          })
          .signers([participant])
          .rpc();
        
        console.log("Study feedback tx:", tx);
      } catch (error) {
        console.error("Study feedback error:", error);
        throw error;
      }
    });

    it("Completes study and distributes rewards", async () => {
      try {
        const initialResearcherBalance = await getAccount(
          provider.connection,
          researcherTokenAccount
        ).then(acc => acc.amount);

        const initialParticipantBalance = await getAccount(
          provider.connection,
          participantTokenAccount
        ).then(acc => acc.amount);

        const tx = await program.methods
          .completeStudy()
          .accounts({
            study: studyPda,
            participant: participantPda,
            researcher: researcherPda,
            researcherTokenAccount: researcherTokenAccount,
            participantTokenAccount: participantTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        console.log("Study completion tx:", tx);

        // Verify token transfers
        const finalResearcherBalance = await getAccount(
          provider.connection,
          researcherTokenAccount
        ).then(acc => acc.amount);

        const finalParticipantBalance = await getAccount(
          provider.connection,
          participantTokenAccount
        ).then(acc => acc.amount);

        const studyAccount = await program.account.study.fetch(studyPda);
        expect(finalResearcherBalance).to.equal(
          initialResearcherBalance - studyAccount.rewardAmount.toNumber()
        );
        expect(finalParticipantBalance).to.equal(
          initialParticipantBalance + studyAccount.rewardAmount.toNumber()
        );

        // Verify study and participant stats
        const participantAccount = await program.account.participant.fetch(participantPda);
        expect(participantAccount.activeStudies).to.equal(0);
        expect(participantAccount.completedStudies).to.equal(1);
        expect(studyAccount.completedParticipants).to.equal(1);
      } catch (error) {
        console.error("Study completion error:", error);
        throw error;
      }
    });

    it("Fails to complete study with insufficient funds", async () => {
      const poorResearcher = Keypair.generate();
      const [poorResearcherPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("researcher"), poorResearcher.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .completeStudy()
          .accounts({
            study: studyPda,
            participant: participantPda,
            researcher: poorResearcherPda,
            researcherTokenAccount: researcherTokenAccount,
            participantTokenAccount: participantTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([poorResearcher])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Error Handling", () => {
    it("Fails to create study with invalid parameters", async () => {
      try {
        await program.methods
          .createStudy(
            "Test Study",
            "Test Description",
            "test_criteria_hash",
            0, // Invalid reward amount
            10
          )
          .accounts({
            study: studyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid reward amount");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Fails to create study with excessive max participants", async () => {
      try {
        await program.methods
          .createStudy(
            "Test Study",
            "Test Description",
            "test_criteria_hash",
            1000,
            2000 // Exceeds maximum limit
          )
          .accounts({
            study: studyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error for excessive max participants");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Token Operations", () => {
    it("Handles token transfer failures gracefully", async () => {
      try {
        // Attempt to transfer more tokens than available
        await program.methods
          .completeStudy()
          .accounts({
            study: studyPda,
            participant: participantPda,
            researcher: researcherPda,
            researcherTokenAccount: researcherTokenAccount,
            participantTokenAccount: participantTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();
        
        expect.fail("Should have thrown an error for insufficient tokens");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Properly handles token account closure", async () => {
      try {
        // Create a temporary token account
        const tempTokenAccount = await createAccount(
          provider.connection,
          admin,
          studyMint.publicKey,
          admin.publicKey
        );

        // Close the account
        await closeAccount(
          provider.connection,
          admin,
          tempTokenAccount,
          admin.publicKey,
          admin
        );

        // Verify account is closed
        try {
          await getAccount(provider.connection, tempTokenAccount);
          expect.fail("Account should be closed");
        } catch (error) {
          expect(error).to.exist;
        }
      } catch (error) {
        console.error("Token account closure error:", error);
        throw error;
      }
    });
  });

  describe("Study Management", () => {
    it("Properly tracks study progress", async () => {
      try {
        const progress = 50; // 50% completion
        
        await program.methods
          .trackStudyProgress(progress)
          .accounts({
            study: studyPda,
            participant: participantPda,
            authority: participant.publicKey,
          })
          .signers([participant])
          .rpc();

        const studyAccount = await program.account.study.fetch(studyPda);
        expect(studyAccount.progress).to.equal(progress);
      } catch (error) {
        console.error("Study progress tracking error:", error);
        throw error;
      }
    });

    it("Handles study completion criteria", async () => {
      try {
        // Set up study with specific completion criteria
        await program.methods
          .createStudy(
            "Criteria Test Study",
            "Test Description",
            "test_criteria_hash",
            1000,
            10
          )
          .accounts({
            study: studyPda,
            researcher: researcherPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([researcher])
          .rpc();

        // Attempt to complete study without meeting criteria
        try {
          await program.methods
            .completeStudy()
            .accounts({
              study: studyPda,
              participant: participantPda,
              researcher: researcherPda,
              researcherTokenAccount: researcherTokenAccount,
              participantTokenAccount: participantTokenAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([researcher])
            .rpc();
          
          expect.fail("Should have thrown an error for unmet completion criteria");
        } catch (error) {
          expect(error).to.exist;
        }
      } catch (error) {
        console.error("Study completion criteria error:", error);
        throw error;
      }
    });
  });
});
