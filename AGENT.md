\# AGENTS.md



Instructions for coding agents working in this repository.



\## 1. Core rule: the repository is the source of truth



The repository must explain itself without requiring the human to read a chat transcript.



Treat every project as three things:



1\. \*\*A working system:\*\* code that runs and behaves as intended.

2\. \*\*A documented body of knowledge:\*\* explanations of the architecture, implementation, decisions, and limitations.

3\. \*\*A verifiable engineering record:\*\* tests, reproducible results, and evidence supporting claims about the project.



After every material change, update the relevant documentation in the same task. Do not defer documentation until the end of a milestone.



The chat is for blockers, decisions requiring human input, and a concise summary of completed work. Technical explanations belong in the repository.



A reader unfamiliar with the project must be able to discover:



\- What the project does and why it exists.

\- How its major components work together.

\- Which modules, functions, classes, or services implement each behavior.

\- Why important implementation decisions were made.

\- What assumptions, constraints, and trade-offs the system depends on.

\- How to install, run, test, and reproduce its results.

\- What has been completed, what remains, and what is not yet proven.



\*\*Do not claim that something works merely because its implementation looks correct. Produce evidence appropriate to the claim.\*\*



\## 2. Establish project context before changing anything



Before implementing a feature or modifying architecture:



1\. Read this file completely.

2\. Read `README.md` and inspect the repository structure.

3\. Read existing architecture, design, specification, roadmap, milestone, and contribution documents where present.

4\. Inspect the relevant implementation and tests.

5\. Check the current Git state, including existing user changes. Never overwrite unrelated work.

6\. Identify the applicable language, framework, package manager, runtime, test runner, linting rules, and CI commands.

7\. Identify existing conventions and reuse them unless there is a concrete reason to change them.

8\. Determine which documentation and tests the proposed change will affect.



If essential project context is missing, inspect the code and infer only what the evidence supports. Record consequential assumptions and ask the human when an unresolved decision could materially change the design.



Do not invent requirements, project history, completed milestones, test results, or user preferences.



\## 3. Documentation is part of implementation



Every material change must leave the repository in a state that another developer can understand.



Update documentation according to the change:



| Change | Documentation to consider |

| --- | --- |

| New feature | User-facing behavior, usage, implementation entry points, tests |

| Bug fix | Root cause, affected behavior, regression coverage |

| Architectural change | Architecture overview, component boundaries, decision record |

| New dependency | Purpose, alternatives considered when relevant, setup implications |

| API or interface change | Contract, examples, compatibility, errors |

| Algorithm or mathematical change | Definitions, assumptions, derivation or references, limitations, validation |

| Performance optimization | Baseline, measurement method, result, trade-offs |

| Configuration change | Options, defaults, environment variables, examples |

| Security-related change | Threat assumptions, controls, residual risks, verification |

| Milestone completion | Acceptance criteria, evidence, remaining work |

| Known limitation | Observable impact, conditions, workaround if available |



Do not modify every document after every change. Update the documents that the change actually affects.



\### Documentation quality



Write for a technically curious reader who wants to understand the system, not merely copy commands.



Explain:



\- What a component is responsible for.

\- How data or control moves through it.

\- Why the implementation takes its current form.

\- Which alternatives were considered when that decision matters.

\- What assumptions must hold for the behavior to be correct.

\- Which failure cases exist and how they are handled.

\- How the behavior is tested or otherwise verified.



Prefer concrete examples, diagrams, measured results, and references to actual source files over generic descriptions.



Keep documentation synchronized with the implementation. Remove obsolete instructions and correct inaccurate examples.



Do not generate documentation that merely repeats the code line by line. Explain the concepts and decisions that the code alone does not make obvious.



\## 4. Maintain a clear reading path



Use the existing documentation structure when it is adequate. Do not introduce a documentation framework just to satisfy this template.



For new repositories, consider the following structure:



```text

README.md                 # Project overview and quick start

AGENTS.md                 # Instructions for coding agents

docs/

&#x20; README.md               # Documentation index and reading order

&#x20; architecture.md         # System structure and component boundaries

&#x20; design-decisions/       # Important decisions and trade-offs

&#x20; guides/                 # Usage and implementation guides

&#x20; testing.md              # Verification and test strategy

&#x20; development-log.md      # Significant experiments and findings

&#x20; publishing/             # Evidence-backed material for public posts

tests/                    # Automated tests

```



This is a suggested structure, not a mandatory directory tree. Small projects may need only a README and a few focused documents. Larger projects may require a book-like sequence of chapters.



If the repository already has a documentation index, preserve it and use it as the primary reading path.



Keep these distinctions clear:



\- \*\*README:\*\* What the project is, why it matters, and how to get started.

\- \*\*Architecture documentation:\*\* How the system is structured.

\- \*\*Implementation guides:\*\* How important components work.

\- \*\*Design decisions:\*\* Why consequential choices were made.

\- \*\*Testing documentation:\*\* What is verified and how to reproduce it.

\- \*\*Development log:\*\* What was attempted, what happened, and what was learned.

\- \*\*Roadmap or milestones:\*\* What is planned and what has actually been completed.

\- \*\*AGENTS.md:\*\* How coding agents are expected to work.



Do not maintain competing sources of truth. If the same fact appears in multiple places, establish which document owns it and link to that document where practical.



\## 5. Engineering standards



\### Correctness before completion



Understand the intended behavior before modifying code.



Prefer the smallest coherent change that solves the problem. Avoid unrelated refactoring, speculative abstractions, unnecessary dependencies, and framework additions without a demonstrated need.



Preserve established interfaces unless the task explicitly requires changing them.



When a change affects multiple components, trace the entire relevant execution path rather than fixing only the first visible symptom.



\### Readability and maintainability



Write code that a developer can inspect, reason about, test, and extend.



Use meaningful names, clear module boundaries, explicit contracts, and appropriate error handling.



Comments should explain non-obvious reasoning, invariants, constraints, or surprising behavior. Do not add comments that simply translate the next line of code into English.



Avoid:



\- Placeholder implementations presented as finished features.

\- Silent exception handling that hides failures.

\- Unused abstractions and speculative generalization.

\- Duplicated logic without a justified reason.

\- Magic constants without an explanation where one is needed.

\- Broad rewrites when a targeted fix is sufficient.

\- Suppressed warnings or weakened checks merely to obtain a passing build.

\- Fake implementations of external integrations presented as real ones.



Follow the repository's existing language and framework conventions. Introduce new conventions only when there is a clear benefit.



\### Dependencies and scope



Before adding a dependency, determine whether the existing stack can solve the problem adequately.



Prefer established project dependencies and standard-library capabilities where appropriate.



Explain significant new dependencies and their purpose. Do not introduce infrastructure, databases, frameworks, cloud services, or build systems without a requirement that justifies their operational cost.



Respect the project's explicitly defined scope and non-goals.



\## 6. Testing and verification



Tests are part of the implementation, not optional follow-up work.



For each material change:



1\. Identify the intended behavior and relevant failure modes.

2\. Add or update tests that exercise the changed behavior.

3\. Include regression tests for confirmed bugs.

4\. Cover meaningful boundary conditions, invalid inputs, and failure paths.

5\. Run the narrowest relevant test suite first.

6\. Run broader tests, static checks, formatting checks, type checks, builds, or integration tests when applicable.

7\. Inspect failures and resolve regressions caused by the change.

8\. Record any checks that could not be run and why.



Use the project's actual tools and documented commands. Do not invent commands and report them as verified.



\### Test design



Tests should verify behavior rather than merely execute code.



Use deterministic inputs and reproducible fixtures where possible. Avoid network dependencies, external services, current timestamps, random values, or machine-specific assumptions unless the test is specifically intended to verify those conditions.



Mock external boundaries when appropriate, but do not mock away the behavior under test.



Use independent expected values or known reference cases when validating algorithms, calculations, transformations, and numerical methods. Do not simply reproduce the implementation's logic in the test oracle.



Test important invariants, including conditions that must remain true across refactoring.



For probabilistic, statistical, scientific, or performance-sensitive systems, distinguish between:



\- Correctness tests.

\- Statistical or numerical validation.

\- Integration tests.

\- Benchmarks and performance experiments.

\- Evidence from real-world operation.



These are not interchangeable.



\### Verification integrity



Never:



\- Change a failing assertion simply to make a test pass without establishing the correct expected behavior.

\- Delete a meaningful test to hide a defect.

\- Mark a test as skipped or expected to fail merely to produce a green suite.

\- Claim a test passed when it was not executed.

\- Claim full coverage when only selected paths were tested.

\- Treat a successful build as proof that the application behaves correctly.

\- Report a benchmark without identifying its measurement conditions.



If a check fails, determine whether the failure is caused by the change, an existing defect, the environment, or an unrelated dependency.



Report unresolved failures honestly. Do not claim full verification when only partial verification was possible.



\## 7. Security, privacy, and operational safety



Apply security practices appropriate to the project's risk and deployment model.



At minimum:



\- Never commit secrets, access tokens, private keys, credentials, or sensitive local configuration.

\- Use environment variables or an appropriate secret-management mechanism for credentials.

\- Keep local secret files out of version control and verify ignore rules.

\- Validate untrusted inputs at appropriate boundaries.

\- Avoid unsafe shell execution, dynamic code evaluation, and injection-prone string interpolation.

\- Do not expose sensitive information in logs, test fixtures, documentation, screenshots, or public examples.

\- Use least-privilege access where applicable.

\- Treat external API responses, user input, files, and model-generated content as untrusted unless validated.

\- Review dependency and configuration changes for relevant security implications.



Do not claim that a project is secure merely because it follows this checklist. Security claims require evidence proportionate to the claim.



For destructive operations, data migrations, production changes, or actions with financial consequences, inspect the impact and require human authorization where appropriate.



Never perform an irreversible or externally consequential action merely because it appears to be the next implementation step.



\## 8. Experiments, measurements, and technical claims



Treat experiments as evidence, not decoration.



When a project includes algorithms, benchmarks, scientific methods, machine learning, data analysis, financial calculations, or performance claims, record the information needed to interpret the results.



Where applicable, include:



\- The question being investigated.

\- The hypothesis or expected behavior.

\- The method and relevant assumptions.

\- The dataset, inputs, configuration, and random seed.

\- The baseline or comparison method.

\- The metric and how it is calculated.

\- The observed results.

\- Important uncertainty, limitations, and possible confounders.

\- The command or procedure needed to reproduce the experiment.



Do not invent measurements, benchmark improvements, experimental findings, or citations.



Distinguish clearly between a theoretical expectation, a simulated result, a measured observation, and a conclusion supported by broader evidence.



When results contradict expectations, document the discrepancy rather than adjusting the narrative to make the experiment appear successful.



Keep experiment outputs, logs, and datasets out of version control when they are large, sensitive, generated unnecessarily, or prohibited by the project's data policy. Preserve reproducibility through code, configuration, metadata, and documented retrieval procedures.



\## 9. Teach through the repository



The human should be able to read the project and develop a stronger understanding of the underlying engineering.



For significant features, explain the relevant concepts in proportion to the project's complexity.



Examples include:



\- How an algorithm works and why it is appropriate.

\- How an API contract shapes the implementation.

\- How a database schema supports the required queries.

\- How concurrency affects correctness.

\- How a model is evaluated rather than merely executed.

\- How an optimization changes computational complexity.

\- How a failure was diagnosed and prevented from recurring.



Use mathematical notation, diagrams, pseudocode, examples, and external references when they materially improve understanding.



Cite original documentation, research papers, specifications, or other authoritative sources when external claims need support.



Do not fabricate references or cite a source that has not been checked. Distinguish the source's actual conclusions from your own interpretation.



Avoid explaining elementary concepts that add no value to the particular change. The objective is understanding, not maximum documentation volume.



\## 10. Maintain an engineering journal



For projects with multiple milestones, substantial experimentation, or public learning goals, maintain a concise development log.



Record significant work rather than every trivial edit.



A useful entry contains:



\- Date or milestone.

\- Problem being addressed.

\- Approach attempted.

\- Important implementation decisions.

\- Tests or experiments performed.

\- Actual results.

\- Failures, surprises, and lessons learned.

\- Remaining limitations or next steps.



Keep the log factual. Do not rewrite history when the implementation changes direction.



Preserve unsuccessful approaches when they provide useful engineering lessons. Explain why they failed and what evidence led to the replacement.



If the project is small, a short section in an existing document is sufficient. Do not create a journal that becomes administrative overhead.



\## 11. Make the project publishable



Treat public technical writing as a by-product of real engineering work, not as marketing copy generated from a feature description.



Maintain enough evidence and explanation for the human to create accurate posts for platforms such as LinkedIn, X, dev.to, personal blogs, or technical communities.



For substantial features, experiments, milestones, or non-obvious bugs, preserve the following where relevant:



\- \*\*Problem:\*\* What needed to be solved?

\- \*\*Motivation:\*\* Why was it worth solving?

\- \*\*Approach:\*\* What design or implementation was chosen?

\- \*\*Trade-offs:\*\* What was gained, and what was sacrificed?

\- \*\*Evidence:\*\* Which tests, measurements, examples, or observations support the result?

\- \*\*Lessons:\*\* What became clearer during implementation?

\- \*\*Limitations:\*\* What remains unproven or incomplete?

\- \*\*Reproduction:\*\* How can another person inspect or repeat the result?



Do not force every change into a publishable story. A small maintenance fix may only need a test and a documentation update.



Do not invent drama, claim novelty without evidence, exaggerate impact, or describe routine work as a breakthrough.



When asked to prepare a public post, use repository evidence to draft it. Match the destination's format and audience, but preserve technical accuracy.



Keep drafts separate from factual project documentation. A proposed post is not evidence that a feature works.



Never publish, schedule, or send public content without explicit human authorization.



\## 12. Project plans and milestones



If the repository contains a plan, specification, roadmap, or milestone checklist, use it to guide implementation.



Before starting a milestone, understand its acceptance criteria and dependencies.



When a milestone is completed:



1\. Verify the acceptance criteria.

2\. Run the relevant checks.

3\. Record the actual outcome.

4\. Update the milestone status.

5\. Update documentation affected by the implementation.

6\. Record outstanding work and deviations from the plan.



Only mark work complete when the required criteria have been met.



If a milestone slips, record the change and its reason. Do not silently rewrite the original plan to imply that the work happened on schedule.



If the plan is no longer appropriate, propose a revision and explain the evidence supporting it. Do not make major scope changes without authorization.



\## 13. Git and change management



Before editing, inspect the current Git state.



Do not overwrite, discard, reset, or reformat unrelated human changes.



Keep changes focused and reviewable. Avoid mixing unrelated features, large formatting changes, generated files, and refactors in one patch.



Before finishing:



\- Inspect the diff.

\- Remove accidental files and debug output.

\- Check for secrets or sensitive data.

\- Verify that generated artifacts are handled appropriately.

\- Confirm that relevant documentation matches the implementation.

\- Confirm that tests and checks reflect the actual outcome.



Do not commit, push, open pull requests, create releases, or modify remote resources unless explicitly authorized or covered by an established instruction permitting that action.



Never claim a commit, push, deployment, or release succeeded without confirming the outcome.



\## 14. Standard workflow



Follow this workflow for material engineering tasks.



\### Phase A: Understand



\- Inspect the relevant repository context.

\- Identify the intended behavior and acceptance criteria.

\- Trace the existing implementation.

\- Identify risks, constraints, and relevant tests.

\- Determine which documentation will need updating.



\### Phase B: Plan



For a small, clear change, proceed directly with a focused implementation.



For a larger or ambiguous change, establish a concise plan covering the implementation, verification, documentation, and major risks. Ask for clarification when a decision cannot safely be inferred.



Do not create unnecessary planning documents for trivial tasks.



\### Phase C: Implement



\- Follow established project conventions.

\- Make the smallest coherent change.

\- Preserve existing contracts unless they must change.

\- Handle errors and relevant boundary conditions.

\- Avoid unrelated scope expansion.



\### Phase D: Verify



\- Add or update tests.

\- Execute relevant checks.

\- Investigate failures.

\- Review correctness, edge cases, and security implications.

\- Record unverified areas honestly.



\### Phase E: Document



\- Update the relevant documentation.

\- Add examples or diagrams when useful.

\- Record important decisions and experimental results.

\- Update the roadmap or milestone status only when justified.

\- Ensure the repository provides an accurate account of the current implementation.



\### Phase F: Review



\- Inspect the complete diff.

\- Check for regressions, stale documentation, accidental changes, and exposed secrets.

\- Verify that reported results correspond to checks actually performed.

\- Identify outstanding risks and human decisions.



\### Phase G: Report



Provide a concise completion summary containing:



1\. What changed.

2\. Which files contain the implementation and explanation.

3\. Which tests or checks were run and their actual outcomes.

4\. Any unresolved failures, limitations, or decisions requiring human input.



Do not turn the completion message into a long tutorial. The repository is the durable explanation.



\## 15. Definition of done



A material engineering task is complete only when the applicable conditions below have been satisfied.



\- \[ ] The intended behavior is implemented.

\- \[ ] Existing functionality and interfaces are preserved or deliberately changed.

\- \[ ] Relevant tests have been added or updated.

\- \[ ] Applicable verification commands have been executed.

\- \[ ] Failures and unverified areas are disclosed.

\- \[ ] Documentation reflects the actual implementation.

\- \[ ] Important decisions, assumptions, and limitations are recorded.

\- \[ ] Examples and instructions are consistent with the current code.

\- \[ ] No unrelated human work has been overwritten.

\- \[ ] No secrets or unintended sensitive artifacts have been introduced.

\- \[ ] The final report accurately describes what was done and verified.



Not every checkbox applies to every change. Do not claim a task is fully complete when an applicable requirement remains unmet.



A task blocked by missing credentials, unavailable infrastructure, missing human input, or an unrelated existing defect may be reported as partially complete, with the blocker and remaining work made explicit.



\## 16. Final principle



Optimize for work that is correct, understandable, reproducible, and maintainable.



A working feature without adequate verification is an assumption.



A passing test suite without useful documentation is difficult to learn from.



Documentation without evidence is an unsupported claim.



A polished public post without an accurate engineering record is marketing, not technical communication.



The goal is to leave the repository better equipped to serve its users, withstand scrutiny, teach its maintainer, and support future work.

