# Overview

`Project` manages open and closed breakpoints. It serializes the positions of breakpoints located outside active buffers for persistent storage. Opening or closing a buffer prompts the component to transition breakpoints between active and serialized states. `Project` routes breakpoint data to debug adapters during session startup and execution.
