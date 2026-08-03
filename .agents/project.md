# Project definition

## Vision

Build a portable, open-source engine that evaluates career documents with deterministic, explainable rules and exposes those results safely to native applications and coding agents.

The project should remain useful when no LLM, API key, account, server, or network connection exists.

## Primary consumers

1. `career` CLI users and coding agents
2. a future native SwiftUI macOS/iOS application
3. future language bindings only when a concrete host requires them

The existing Django project at `../resume-ai` remains an independent application and behavioral reference. Sharing a runtime is not a goal.

## User value

- private, local processing of sensitive resumes and job descriptions
- inspectable evidence behind scores and gaps
- reproducible results rather than prompt-dependent judgments
- conservative matching that does not invent skill equivalence
- a structured tool that agents can call without trusting the agent to calculate scores

## Product principles

- deterministic first
- evidence before recommendation
- parser uncertainty is explicit
- no hidden network behavior
- no AI requirement or provider call in the core
- optional assisted fields remain separate from deterministic truth
- stable machine-readable contracts
- minimal dependencies and boring implementation
- false negatives are safer than fabricated equivalence

## Core vocabulary

- **source input**: caller-supplied resume or job text and metadata
- **normalized document**: bounded structured facts extracted deterministically from source input
- **check**: one versioned scoring rule with score, status, explanation, and evidence
- **evidence**: a bounded source-grounded reference supporting a result
- **warning**: uncertainty or limitation that a consumer must not hide
- **section evaluation**: the preserved Phase 1 core-header coverage operation
- **resume analysis**: the full deterministic readiness checks, category scores, evidence, findings, actions, and limitations
- **job normalization**: source-grounded lexical classification of caller-supplied job-description text with explicit confidence and uncertainty
- **match**: deterministic comparison between independently reproduced resume/job baselines, with conservative equivalence, confidence-aware evidence, and bounded recommendation gates
- **capability**: a versioned operation advertised as available or planned
- **adapter**: CLI, Swift binding, or other boundary depending on the core
- **external proposal**: bounded untrusted source-grounded structured data submitted by an opted-in host for core validation
- **assisted document**: a separately labeled view containing accepted external values without replacing the deterministic baseline
- **reviewed assisted suggestion**: bounded externally written advisory review text tied to a current deterministic analysis action and source occurrence; it never becomes a resume candidate or scoring input
- **reviewed assisted replacement**: bounded externally written exact before/proposed-after review value tied to a current deterministic analysis action and source occurrence; it supports diff display only and never becomes a resume candidate or scoring input

## Non-goals

Until a phase explicitly changes them:

- LLM prompts or provider clients inside the root core library
- web APIs or hosted services
- databases or user accounts
- UI frameworks
- PDF/DOCX parsing inside the core
- job-board scraping or arbitrary URL fetching
- automatic resume rewriting
- claims of ATS compatibility without evidence
- fuzzy or embedding-based skill equivalence
- analytics, telemetry, or remote feature flags
- plugin frameworks

## Success criteria for the first usable release

A coding agent or local program can submit bounded plain-text resume and job inputs, receive versioned deterministic JSON with scores/evidence/warnings, reproduce the same output, and understand every active limitation without network access. An opted-in host may additionally submit an external proposal for deterministic grounding validation while retaining the unchanged local baseline.
