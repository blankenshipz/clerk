[#](#) Clerk

## Usecase

You have a large amount of poorly organized files that fall into a set number of tags or categories
and you want to automate the process of associating with those tags so they can be better organized.

## About

Clerk uses LLMs to magically provide context about your files!

Clerk works on the current directory and requires a YAML config. The default name for this
file is `clerk.yml` and it is expected in the working directory.

### Example Config:

```clerk.yml
categories:
  genre:
     - autobiography
     - fantasy
     - historical fiction
     - non fiction
     - romance
     - science fiction
```

### How's the LLM magic sprinkled on top?

For each file recursively walking down from the current working directory we construct a prompt
for the LLM asking it to attribute one of each of the category values to the file based on the name
of the file and some of the content of the file.

* The LLM has a hard limit on the number of tokens; this impacts how many categories and how much file content can be sent as part of the prompt.

The amount of file content sent as part of the prompt can be increased or decreased. If you decrease it 
you'll have more room for category values in the prompt. If you increase it you _may_ have more accuracy.

### Output

Currently clerk outputs a JSON line for each file with the path to the file, and a key, value for each category and the prediction for the category value
from the LLM

```
{ "path": "/some/long/path/book1.pdf", "genre": "fiction" }
{ "path": "/some/long/path/book2_2022-01-03-harry-potter.pdf", "genre": "fiction" }
```

## Currently Supported File Types

* Text
* PDF

## Usage

Currently clerk only supports the OpenAI GPT-4 model; you'll have to and
to that model and an API key in the environment variable `OPENAI_API_KEY`

```
Usage: clerk [OPTIONS]

Options:
  -m, --max-read-length <MAX_READ_LENGTH>
          Maximum length of content to read from files for matching [default: 10000]
  -e, --exclude-file-type <EXCLUDE_FILE_TYPE>
          Excluded File Type [default: zip xlsx yml]
  -c, --config-file <CONFIG_FILE>
          Location of Configuration file that defines file categories [default: clerk.yml]
  -h, --help
          Print help
  -V, --version
          Print version
```
