# PRD: RustyJS-UI – Next Steps (Performance + Native + DX)

## 1. Overview

RustyJS-UI should become:
* **Faster** at runtime and build time
* **More complete** with robust native components
* **More ergonomic** for developers via better APIs and tooling

This PRD defines priorities, scope, milestones, and success metrics for the next 1–2 release cycles.

## 2. Goals

### G1 — Performance
* Reduce initial render and interaction latency
* Reduce bundle size and avoid unnecessary re-renders
* Improve perceived performance for large lists and complex trees

### G2 — Native Components Coverage
* Add missing foundational components
* Improve existing component behavior/accessibility consistency
* Ensure “native feel” across platforms/environments

### G3 — Developer Experience (DX)
* Simplify component and API usage
* Improve type safety and error messages
* Improve docs, examples, and debugging tools

## 3. Non-Goals (for this phase)
* Full redesign of component visual language
* Breaking API rewrite of the whole library
* Supporting legacy runtimes that significantly block performance work

## 4. User Personas
* **App Developer**: Wants fast, predictable components with minimal boilerplate
* **Library Integrator**: Wants stable APIs and clear migration paths
* **Maintainer/Contributor**: Wants clear architecture and testable modules

## 5. Problem Statement
Current friction appears in:
* **Performance hotspots** in complex/large UIs
* **Gaps in component coverage** and inconsistency among existing components
* **DX pain** around native APIs, docs discoverability, and integration patterns

---

## 6. Scope & Initiatives

### Initiative A: Performance

#### A1. Render Performance
* Audit re-render patterns in core components
* Add memoization/selective updates where safe
* Virtualization for long lists/tables

#### A2. Bundle/Build Performance
* Tree-shaking audit + dead-code elimination opportunities
* Split heavy modules and lazy-load where possible
* Optimize build config for faster local feedback loops

#### A3. Instrumentation
* Add lightweight perf benchmarks for key scenarios
* Track “before/after” for TTI-like metric, render time, and memory

### Initiative B: Native Components

#### B1. New Native Components (Priority Order)
1. **NativeList** (virtualized, performant)
2. **NativeModal** (focus trap, keyboard-safe)
3. **NativeSelect / NativeCombobox**
4. **NativeToast** / lightweight notifications
5. **NativeTabs** with a11y support

#### B2. Improve Existing Components
* Standardize controlled/uncontrolled behavior
* Improve keyboard interactions and ARIA patterns
* Normalize styling/theming APIs and prop naming
* Harden edge cases with better test coverage

### Initiative C: Native APIs for DX

#### C1. API Layer
Add higher-level native APIs for common app patterns:
* Navigation helpers
* Storage abstraction
* Event bridge utilities
* *Keep low-level escape hatches for advanced use*

#### C2. Developer Tooling
* Better TypeScript inference and helper types
* Runtime warnings with actionable guidance (dev-only)
* Optional lint rules/codemods for best practices

#### C3. Documentation DX
* “Start here” quickstart paths by use case
* Copy-paste examples for each major component/API
* Migration guides for changed/improved APIs

---

## 7. Requirements

### Functional
* New components ship with documented props, examples, and accessibility behavior
* Existing components pass compatibility checks for current consumers
* Native APIs include clear contracts and typed signatures

### Non-Functional
* No major regressions in bundle size
* Performance benchmarks show measurable gains
* DX improvements reduce setup and integration time

## 8. Success Metrics (KPIs)
* **Runtime**: 20–30% reduction in expensive render scenarios
* **Bundle**: 10–15% reduction in core package footprint
* **DX**:
  * 30% faster “first feature shipped” in internal onboarding
  * Fewer integration-related issues/bugs
  * Higher docs usage satisfaction (survey or feedback proxy)

## 9. Milestones

### Milestone 1: Baseline + Quick Wins (Weeks 1–3)
* Perf audit + benchmark harness
* Fix top 3 re-render hotspots
* Draft API standards for component consistency

### Milestone 2: Component Expansion (Weeks 4–7)
* Ship first 2 new native components
* Improve top 3 existing components (a11y + behavior consistency)
* Publish revised docs structure

### Milestone 3: Native APIs + Stabilization (Weeks 8–10)
* Ship first native API pack
* Add TS/DX helpers and warnings
* Final perf validation + release notes + migration notes

---

## 10. Risks & Mitigations

* **Risk**: Performance optimizations introduce subtle behavior changes
  * **Mitigation**: Snapshot + interaction regression tests; staged rollout
* **Risk**: API improvements increase maintenance complexity
  * **Mitigation**: Clear layering (public API vs internal primitives)
* **Risk**: Component expansion slows quality
  * **Mitigation**: Strict definition of done and shared test templates

## 11. Definition of Done (DoD)
A work item is done when:
* Code merged with tests
* Docs/examples updated
* Performance and a11y checks pass for affected areas
* Changelog/migration notes included (if behavior changed)

