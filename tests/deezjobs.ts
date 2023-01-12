import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Deezjobs } from "../target/types/deezjobs";

describe("deezjobs", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Deezjobs as Program<Deezjobs>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
