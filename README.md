Notice
======

Traits and some implementations for waiting and notifying.

## Introduction

A common task that comes up in development is waiting for an event to happen. As
it turns out, there are many different ways to accomplish this goal.

## Implementations

### EventFd

Linux provides a nearly exact fit for implementing notice: `EventFd`. This
implementation is available in the `notice-eventfd` crate.

### Pipes

Non-Linux Unix systems don't provide `EventFd`, so there is an alternate
implementation on top of pipes in `notice-pipes`.

### Other Operating Systems

_to be implemented_
