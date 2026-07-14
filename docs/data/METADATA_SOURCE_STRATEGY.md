# Metadata and Asset Source Strategy

## Recommendation for the MVP

Use a hand-reviewed, static, versioned catalog. Do not build a database server and do not synchronize by scraping a community wiki.

The repository should contain normalized records that are updated through pull requests whenever a new Arcaea game version or meaningful content update is released.

## Source hierarchy

Use sources in this order:

1. **Official first-party sources** for release facts, names, dates, and public descriptions: the official Arcaea website, official announcements, and information visible in the game where recording it is lawful.
2. **Licensed or explicitly permitted community sources** for structured facts or media.
3. **Community wikis as discovery and cross-checking aids only.** Do not copy their prose, page structure, images, or bulk dataset without a compatible license and attribution path.
4. **Maintainer-authored summaries** written in original language from verified facts.

Conflicting facts must remain unresolved until a source is reviewed. Never silently choose a convenient value.

## MVP entities

Start with these normalized entities:

- `release`
- `song`
- `chart`
- `pack`
- `partner`
- `storyEntry`
- `gameTopic`
- `sourceRecord`
- `assetRecord`

The MVP should not promise a complete catalog before the data workflow is proven. A smaller, internally consistent dataset is better than a large unverifiable dump.

## Suggested fields

### Source record

```json
{
  "id": "source-official-release-6-0",
  "sourceType": "official-site",
  "url": "https://example.invalid",
  "title": "Release announcement",
  "retrievedAt": "2026-07-14T00:00:00Z",
  "gameVersion": "6.0.0",
  "licenseStatus": "facts-only",
  "reviewStatus": "verified",
  "reviewedBy": "Dyu20705",
  "notes": "Original prose and media are not redistributed."
}
```

### Asset record

```json
{
  "id": "asset-partner-placeholder-001",
  "path": "/assets/partners/placeholder-001.avif",
  "kind": "partner-image",
  "owner": "Example creator",
  "license": "CC-BY-4.0",
  "permissionEvidence": "docs/legal/permissions/example.md",
  "sourceUrl": "https://example.invalid",
  "sha256": "..."
}
```

Every committed image must have an asset record. `licenseStatus: unknown` is not sufficient for publication.

## Versioned layout

```text
data/
  schema/
    song.schema.json
    chart.schema.json
    pack.schema.json
    partner.schema.json
    story-entry.schema.json
    game-topic.schema.json
    source-record.schema.json
    asset-record.schema.json
  catalog/
    songs.json
    charts.json
    packs.json
    partners.json
    story.json
    game-topics.json
  releases/
    6.0.0.json
  provenance/
    sources.json
    assets.json
```

## Update workflow per game version

1. Open a release-update issue.
2. Record official source links and the target game version.
3. Add or modify normalized records.
4. Validate schemas, IDs, references, duplicate keys, and asset permissions.
5. Produce a generated catalog summary and diff.
6. Review content accuracy and original wording.
7. Run accessibility, image, and link checks.
8. Merge and publish a versioned static snapshot.

This is a deliberate manual synchronization process. Automation validates and packages data; it does not decide truth.

## Image policy

For the MVP:

- use only original, openly licensed, or explicitly permitted images;
- do not assume that an image being publicly viewable makes redistribution legal;
- do not hotlink third-party images by default;
- retain permission evidence, author, source, license, and checksum;
- generate AVIF/WebP variants and responsive sizes during the build;
- provide meaningful alt text or mark decorative images appropriately.

When permission is unclear, use a designed placeholder and link to the official source rather than copying the image.

## Security and privacy

Static public metadata is low risk, but the pipeline still needs:

- schema and content-size limits;
- URL protocol allowlists;
- safe Markdown/rendering rules;
- no secrets or private contributor data;
- no arbitrary HTML in content files;
- dependency and action pinning;
- generated output checks before deployment.

This document is a technical content policy, not legal advice.
