"""Pure Sparkle feed bytes and signatures shared by release preparation and QA."""
import base64
import re
import xml.etree.ElementTree as ET


class ReleaseError(ValueError):
    pass


SPARKLE = "http://www.andymatuschak.org/xml-namespaces/sparkle"
MAX_FEED_BYTES = 256 * 1024
ET.register_namespace("sparkle", SPARKLE)
# Sparkle 2.10.0 common_cli/Signing.swift. Sign the exact XML bytes, then append
# this block; it is not part of the signed content. Never reserialize signed XML.
SIGNING_BLOCK = re.compile(
    rb"<!-- sparkle-signatures:\nedSignature: ([A-Za-z0-9+/]{86}==)\nlength: ([1-9][0-9]{0,5})\n-->\n\Z"
)


def feed_content(manifest, artifact, package_signature):
    rss = ET.Element("rss", {"version": "2.0"})
    channel = ET.SubElement(rss, "channel")
    ET.SubElement(channel, "title").text = "Beni Buddy"
    ET.SubElement(channel, "link").text = manifest["release_notes_url"]
    ET.SubElement(channel, "description").text = "Updates für Beni Buddy."
    item = ET.SubElement(channel, "item")
    ET.SubElement(item, "title").text = f"Beni Buddy {manifest['version']}"
    ET.SubElement(item, f"{{{SPARKLE}}}version").text = str(manifest["build_number"])
    ET.SubElement(item, f"{{{SPARKLE}}}shortVersionString").text = manifest["version"]
    ET.SubElement(item, f"{{{SPARKLE}}}minimumSystemVersion").text = artifact["requirements"]["minimum_os_version"]
    if manifest["channel"] == "beta":
        ET.SubElement(item, f"{{{SPARKLE}}}channel").text = "beta"
    # No unsigned external HTML release notes. All presentation text is signed.
    ET.SubElement(item, "description").text = artifact["requirements"]["description"]
    ET.SubElement(item, "enclosure", {
        "url": artifact["url"], "length": str(artifact["size_bytes"]),
        "type": "application/octet-stream", f"{{{SPARKLE}}}edSignature": package_signature,
    })
    ET.indent(rss, space="  ")
    return ET.tostring(rss, encoding="utf-8", xml_declaration=True) + b"\n"


def sign_feed(content, key):
    if not 0 < len(content) < MAX_FEED_BYTES - 256:
        raise ReleaseError("Appcast überschreitet die Grössengrenze.")
    signature = base64.b64encode(key.sign(content))
    return content + b"<!-- sparkle-signatures:\nedSignature: " + signature + b"\nlength: " + str(len(content)).encode() + b"\n-->\n"


def verify_feed(data, trusted):
    if not 0 < len(data) <= MAX_FEED_BYTES:
        raise ReleaseError("Appcast überschreitet die Grössengrenze.")
    block = SIGNING_BLOCK.search(data)
    if block is None:
        raise ReleaseError("Appcast hat keinen gültigen Sparkle-Signaturblock.")
    content = data[:block.start()]
    if len(content) != int(block[2]):
        raise ReleaseError("Signierte Appcast-Länge stimmt nicht.")
    trusted.verify(base64.b64decode(block[1], validate=True), content)
    # No XML parsing before successful authentication. Canonical content is
    # compared separately against the already authenticated release candidate.
    return content
