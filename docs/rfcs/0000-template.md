# RFC Template

**Title:** [Short descriptive title]

**Author(s):** [Name(s) and GitHub handle(s)]

**Status:** Draft

**Created:** YYYY-MM-DD

**Discussion:** [Link to GitHub Discussion or PR]

---

## Summary

[One paragraph explaining the proposal in plain language. What problem does this solve? What is the proposed solution?]

---

## Motivation

[Why are we doing this? What use cases does it support? What is the expected outcome?]

### Problem Statement

[Detailed description of the problem. Include concrete examples if possible.]

### Current State

[How does the system work today? What are the limitations?]

---

## Design

[Detailed technical design. This is the core of the RFC.]

### Overview

[High-level architecture diagram or description]

### API / Interface Changes

[Show the new public API, config options, CLI flags, etc.]

```rust
// Example code showing new API
```

### Configuration Changes

[If applicable, show config.json changes]

```json
{
  "new_field": "value"
}
```

### Data Model Changes

[If applicable, describe schema changes]

### Migration Strategy

[How do existing users migrate? Breaking changes? Deprecation path?]

---

## Implementation Plan

[Break down into phases/tasks]

### Phase 1: [Name]

- [ ] Task 1
- [ ] Task 2

### Phase 2: [Name]

- [ ] Task 1
- [ ] Task 2

---

## Alternatives Considered

[What other approaches were considered? Why were they rejected?]

### Alternative 1: [Name]

[Description and why not chosen]

### Alternative 2: [Name]

[Description and why not chosen]

---

## Drawbacks

[What are the downsides? Complexity, performance, maintenance burden?]

---

## Prior Art

[How do other projects solve this? Links to relevant implementations.]

---

## Unresolved Questions

[Open questions that need discussion before implementation]

---

## Testing Strategy

[How will this be tested? Unit tests, integration tests, manual testing?]

---

## Documentation Impact

[What docs need updating? AGENTS.md, CONFIGURATION.md, README.md, etc.]

---

## Security Considerations

[Any security implications? Permissions, network access, data handling?]

---

## Performance Impact

[Expected performance implications. Benchmarks if available.]

---

## Rollout Plan

[How will this be released? Feature flag? Gradual rollout?]

---

## Appendix

[Additional context, diagrams, references]