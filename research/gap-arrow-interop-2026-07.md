<!-- [GPT-5.6] sq-vwnzh — first-read gap record; no work-box timings committed. -->

# Gap record — Arrow/tabular result export interop (2026-07)

**Axis:** SPARQL SELECT result export into a tabular representation.  
**Verdict:** harness delivered; pinned workloads are `RESULT-SET-EQUAL` under the
hermetic oracle. No canonical quiet-box timing has been recorded.

The sparq column executes `sparq_arrow::to_record_batch` and reads RDF terms back from
the resulting Arrow struct columns. The pyoxigraph column iterates its serialized
query solutions. RDF 1.1 simple literals are normalized to explicit `xsd:string`
before comparison, and equality is an exact multiset comparison rather than a row
count. Only after every workload agrees may the runner start timing.

The timing envelope carries `loose/in-process`: pyoxigraph runs in-process, while the
sparq column is process-inclusive unless a prebuilt example is supplied. Missing
tools produce missing columns, never placeholder measurements. Consequently this
panel establishes result interoperability and a first-read export axis; it does not
support a matched-throughput claim.
