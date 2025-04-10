# Tutorial

## Motivation

The main use-case for `tripsu` is to be used in combination with other CLI tools
up- and downstream via piping. Let us assume that we're running a SPARQL query
on a large graph and we would like to pseudonymize some of the triples. This is
how the flow should look like:

```shell
curl <sparql-query> | tripsu pseudo -x index.nt -r rules.yaml > pseudo.nt
```

For this flow to stream data instead of loading everything into memory, note
that an indexing step is required to allow the pseudonymization to run on a
stream without loading the graph into memory.

## Example

There are three possible ways to pseudonymize RDF triples:

1. Pseudonymize the URI of nodes with `rdf:type`.
2. Pseudonymize values for specific subject-predicate combinations.
3. Pseudonymize any value for a given predicate.

By combining these, can process an RDF file with sensitive information:

```ntriples
<http://example.org/Alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://xmlns.com/foaf/0.1/holdsAccount> <http://example.org/Alice-Bank-Account> .
<http://example.org/Alice-Bank-Account> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/OnlineAccount> .
<http://example.org/Alice-Bank-Account> <http://schema.org/name> "my_account32" .
<http://example.org/Alice-Bank-Account> <http://schema.org/accessCode> "secret-123" .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```


into a pseudonymized file where the sensitive information such as people's names, personal and
secret information is hashed to protect privacy:


```ntriples
<http://example.org/af321bbc> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/af321bbc> <http://xmlns.com/foaf/0.1/holdsAccount> <http://example.org/bs2313bc> .
<http://example.org/bs2313bc> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/OnlineAccount> .
<http://example.org/bs2313bc> <http://schema.org/name> "pp54r32" .
<http://example.org/bs2313bc> <http://schema.org/accessCode> "asfnd223" .
<http://example.org/af321bbc> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

The next subsections break down each of the three pseudonymization approaches to
better understand how they operate.  

> [!TIP]
> All the examples use full URIs, but the tripsu config supports prefixes too! [Click here](https://github.com/sdsc-ordes/tripsu/blob/main/tests/data/rules.yaml) for an example.

### 1. Pseudonymize the URI of nodes with `rdf:type`


Given the following config:

```yaml
nodes:
  of_type:
  - "<http://xmlns.com/foaf/0.1/Person>"
```

The goal is to pseudonymize all instaces of `rdf:type` Person. The following
input file:

```ntriples
<http://example.org/Alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
```

Would become:

```ntriples
<http://example.org/af321bbc> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
```


### 2. Pseudonymize values for specific subject-predicate combinations

Given the following config:

```yaml
objects:
  on_type_predicate:
    "<http://xmlns.com/foaf/0.1/Person>":
    - "<http://schema.org/name>"
```

The goal is to pseudonymize only the instances of names when they're associated
to Person. The following input file:

```ntriples
<http://example.org/Alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

Would become:

```ntriples
<http://example.org/Alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

### 3. Pseudonymize any value for a given predicate


Given the following config:

```yaml
objects:
  on_predicate:
  - "<http://schema.org/name>"
```

The goal is to pseudonymize any values associated to name. The following input
file:

```ntriples
<http://example.org/Alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

Would become:

```ntriples
<http://example.org/Alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "38a3dd71" .
```
