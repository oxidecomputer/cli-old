#!/bin/bash
set -e
set -o pipefail

COMMIT=$GITHUB_SHA

filebase='https://buildomat.eng.oxide.computer/public/file'
commitbase="$filebase/oxidecomputer/cli/build-illumos/$COMMIT"


# Wait for the binary.
start=$SECONDS
while :; do
        if (( SECONDS - start > 3600 )); then
                printf 'timed out waiting for artefact\n' >&2
                exit 1
        fi

        rm -f /tmp/oxide-x86_64-unknown-illumos.sha256
        if ! curl -fSsL -o /tmp/oxide-x86_64-unknown-illumos.sha256 \
            "$commitbase/oxide.sha256.txt"; then
                sleep 5
                continue
        fi

        rm -f /tmp/oxide-x86_64-unknown-illumos.gz
        if ! curl -fSsL -o /tmp/oxide-x86_64-unknown-illumos.gz \
            "$commitbase/oxide.gz"; then
                sleep 5
                continue
        fi

        rm -f /tmp/oxide-x86_64-unknown-illumos
        if ! gunzip /tmp/oxide-x86_64-unknown-illumos.gz; then
                rm -f /tmp/oxide-x86_64-unknown-illumos
                rm -f /tmp/oxide-x86_64-unknown-illumos.gz
                sleep 5
                continue
        fi

        exp=$(</tmp/oxide-x86_64-unknown-illumos.sha256)
        hav=$(sha256sum /tmp/oxide-x86_64-unknown-illumos | awk '{ print $1 }')

        if [[ "$exp" != "$hav" ]]; then
                rm -f /tmp/oxide-x86_64-unknown-illumos
                printf 'ERROR: hash %s != %s\n' "$exp" "$hav"
                sleep 5
                continue
        fi

        break
done

set -x

export VERSION=`toml get $(pwd)/Cargo.toml package.version | jq -r .`
export BUILDDIR="$(pwd)/releases/cli/v${VERSION}"

mkdir -p "$BUILDDIR"

export NAME="oxide-x86_64-unknown-illumos"

mkdir - p "$(pwd)/cross"

export README="$(pwd)/cross/README.md"

# Move the files into the right directory.
mv /tmp/${NAME} ${BUILDDIR}/${NAME}
md5sum ${BUILDDIR}/${NAME} > ${BUILDDIR}/${NAME}.md5;
sha256sum ${BUILDDIR}/${NAME} > ${BUILDDIR}/${NAME}.sha256;
echo -e "### x86_64-unknown-illumos\n\n" >> ${README};
echo -e "\`\`\`console" >> ${README};
echo -e "# Export the sha256sum for verification." >> ${README};
echo -e "\$ export OXIDE_CLI_SHA256=\"`cat ${BUILDDIR}/${NAME}.sha256 | awk '{print $1}'`\"\n\n" >> ${README};
echo -e "# Download and check the sha256sum." >> ${README};
echo -e "\$ curl -fSL \"https://dl.oxide.computer/releases/cli/v${VERSION}/${NAME}\" -o \"/usr/local/bin/oxide\" \\" >> ${README};
echo -e "\t&& echo \"\$${OXIDE_CLI_SHA256}  /usr/local/bin/oxide\" | sha256sum -c - \\" >> ${README};
echo -e "\t&& chmod a+x \"/usr/local/bin/oxide\"\n\n" >> ${README};
echo -e "\$ echo \"oxide cli installed!\"\n" >> ${README};
echo -e "# Run it!" >> ${README};
echo -e "\$ oxide -h" >> ${README};
echo -e "\`\`\`\n\n" >> ${README};

set -x
