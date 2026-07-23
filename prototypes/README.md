# Prototypes

Throwaway experiments used to answer a design or technical question. **Not production code.**

- Each prototype gets its own subdirectory, named for the question it's answering (e.g., `prototypes/can-we-stream-graph-updates/`).
- Prototypes are disposable: no test coverage, no code review bar, no expectation of reuse.
- If a prototype's approach is validated, the real implementation is written fresh in the eventual application codebase — do not promote prototype code directly.
- Delete prototype directories once their question is answered and the finding is recorded (in an ADR or research doc).
