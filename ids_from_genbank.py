import gzip

from glob import iglob
from multiprocessing import Pool
from sys import stderr

def print_taxa(fn):
    result = []
    try:
        with gzip.open(fn) as f:
            accessions = None
            tax = None
            for line in f:
                try:
                    if line.strip() == "//":
                        accession = None
                        tax = None
                        continue
                    if line.startswith(b"ACCESSION"):
                        accessions = line.strip().split()[1:]
                    if b"taxon:" in line:
                        try:
                            tax = int(line.split(b"taxon:")[1].split(b'"')[0])
                        except Exception as e:
                            tax = None
                            accessions = None
                    if accessions and tax:
                        for accession in accessions:
                            if b"-" in accession:
                                a1, a2 = accession.split(b"-")
                                result.append(f"{a1.decode('utf8')}\t{tax}")
                                result.append(f"{a2.decode('utf8')}\t{tax}")
                            else:
                                result.append(f"{accession.decode('utf8')}\t{tax}")
                        accessions = None
                        tax = None
                except Exception as e:
                    print(fn, e, file=stderr)
    except Exception as e:
        print(fn, e, file=stderr)
    return result


def gb():
    for i, fn in enumerate(iglob("**", recursive=True)):
        if i % 1000 == 0:
            print(i, file=stderr)
        if fn.endswith(".gz"):
            yield fn


if __name__ == "__main__":
    fs = gb()
    with Pool(32) as p:
        for lines in p.imap_unordered(print_taxa, fs):
            if lines:
                for line in lines:
                    print(line)
