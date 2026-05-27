# WebGPU on Safari

I'm not about to write a bug report for webkit of there are existing ones. But to summarize, the problem lies in webkit not following the webgpu standard correctly?

Yes — to the best of what I can see, this is a WebKit conformance gap, not iced or wgpu doing something off-spec. Three things support that:

The WebGPU spec explicitly permits the mappedAtCreation: true → write → unmap → copyBufferToBuffer pattern, and the gpuweb group's own recommendation literally describes this as how a staging belt should work.
The exact same wgpu command stream works in Chrome (Dawn) and Firefox (wgpu-core) but raises a generic Validation failure. at Queue::submit in WebKit. When two of three major implementations accept identical, spec-sanctioned commands and the third rejects them, the burden of proof is on the third.
WebKit already has a track record of similar Queue::submit validation regressions around mapping (e.g. the philn/WebKit "Invalidate all command buffers on submit" patch, bug #266793 on onSubmittedWorkDone ordering, #267115 on map-copy optimization, etc.).
A couple of caveats worth keeping in the back of your head before you commit to that framing:

I haven't actually traced which WebKit validation rule is firing. The "Validation failure." message is the generic catch-all that WebKit emits for any failed command-buffer validation, so the exact root cause inside WebKit isn't pinned down yet — it could be the staging-belt pattern itself, or something more specific like a size/alignment/ordering quirk that Dawn and wgpu-core happen to be lenient about. A minimal vanilla-JS repro would be what nails it down.
It's also worth checking bugs.webkit.org and the WebKit/WebKit PR list before writing anything new — the [WebGPU]-tagged bug list there changes quickly, and there's a decent chance someone has already filed it. If they have, you can just subscribe and link the iced workaround to that bug ID. If not, you don't have to write the bug yourself; you can also just file a wgpu issue describing the symptom and let the wgpu maintainers (who have direct contact with the WebKit WebGPU team) escalate it.
So: yes, the framing is "WebKit isn't following the WebGPU standard here", with the small honest qualifier that nobody has yet pinned down exactly which rule it's violating.