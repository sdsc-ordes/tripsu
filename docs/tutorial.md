# Tutorial

## Motivation

The main use-case for `tripsu` is to be used in combination with other CLI tools up-
and downstream via piping. Let us assume that we're running a SPARQL query on a
large graph and we would like to pseudonymize some of the triples. This is how
the flow should look like:

```shell
curl <sparql-query> | tripsu pseudo -i index.nt -c config.yaml > pseudo.nt
```

For this flow to stream data instead of loading everything into memory, note that 
an indexing step is required to allow the pseudonymization to run on a stream without
loading the graph into memory.

## Example

There are three possible ways to pseudonymize RDF triples:

1. Pseudonymize the URI of nodes with `rdf:type`.
2. Pseudonymize values for specific subject-predicate combinations.
3. Pseudonymize any value for a given predicate.

By using all three ways together, we're able to get an RDF file with sensitive
information:

<details>
    <summary><b>Click to show input</b></summary>

```ntriples
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://xmlns.com/foaf/0.1/holdsAccount> <http://example.org/Alice-Bank-Account> .
<http://example.org/Alice-Bank-Account> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/OnlineAccount> .
<http://example.org/Alice-Bank-Account> <http://schema.org/name> "my_account32" .
<http://example.org/Alice-Bank-Account> <http://schema.org/accessCode> "secret-123" .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

</details>

And pseudonymize the sensitive information such as people's names, personal and
secret information while keeping the rest as is:

<details>
    <summary><b>Click to show output</b></summary>

```
<http://example.org/af321bbc> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/af321bbc> <http://xmlns.com/foaf/0.1/holdsAccount> <http://example.org/bs2313bc> .
<http://example.org/bs2313bc> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/OnlineAccount> .
<http://example.org/bs2313bc> <http://schema.org/name> "pp54r32" .
<http://example.org/bs2313bc> <http://schema.org/accessCode> "asfnd223" .
<http://example.org/af321bbc> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

</details>

The next subsections break down each of the three pseudonymization approaches to
better understand how they operate.

### 1. Pseudonymize the URI of nodes with `rdf:type`

<details>
    <summary><b>Click to show</b></summary>

Given the following config:

```yaml
replace_uri_of_nodes_with_type:
  - "http://xmlns.com/foaf/0.1/Person"
```

The goal is to pseudonymize all instaces of `rdf:type` Person. The following
input file:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
```

Would become:

```
<http://example.org/af321bbc> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
```

</details>

### 2. Pseudonymize values for specific subject-predicate combinations

<details>
    <summary><b>Click to show</b></summary>

Given the following config:

```yaml
replace_values_of_subject_predicate:
  "http://xmlns.com/foaf/0.1/Person":
    - "http://schema.org/name"
```

The goal is to pseudonymize only the instances of names when they're associated
to Person. The following input file:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

Would become:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

</details>

### 3. Pseudonymize any value for a given predicate

<details>
    <summary><b>Click to show</b></summary>

Given the following config:

```yaml
replace_value_of_predicate:
  - "http://schema.org/name"
```

The goal is to pseudonymize any values associated to name. The following input
file:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "Alice" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "Bank" .
```

Would become:

```
<http://example.org/Alice> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Person> .
<http://example.org/Alice> <http://schema.org/name> "af321bbc" .
<http://example.org/Bank> <http://www.w3.org/2000/01/rdf-schema#type> <http://xmlns.com/foaf/0.1/Organization> .
<http://example.org/Bank> <http://schema.org/name> "38a3dd71" .
```

</details>
