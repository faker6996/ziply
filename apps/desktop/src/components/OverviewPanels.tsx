import type { AppOverview, ArchiveCapabilities } from '../app/types'

interface OverviewPanelsProps {
  overview: AppOverview
  capabilities: ArchiveCapabilities
}

export function OverviewPanels({ overview, capabilities }: OverviewPanelsProps) {
  const formatSupportRows = [
    {
      format: 'zip',
      create: 'Yes',
      extract: 'Yes',
      notes: 'Deflate compression. Password-protected ZIP creation is supported. AES extraction is supported when reading ZIP archives.',
    },
    { format: 'tar', create: 'Yes', extract: 'Yes', notes: 'Pure tar format.' },
    { format: 'tar.gz', create: 'Yes', extract: 'Yes', notes: 'Gzip-compressed tar.' },
    { format: 'tar.bz2', create: 'Yes', extract: 'Yes', notes: 'Bzip2-compressed tar.' },
    { format: 'tar.xz', create: 'Yes', extract: 'Yes', notes: 'XZ-compressed tar.' },
    {
      format: 'xz',
      create: 'Yes',
      extract: 'Yes',
      notes: 'Basic xz stream. Compression supports exactly one file.',
    },
    {
      format: 'bz2',
      create: 'Yes',
      extract: 'Yes',
      notes: 'Basic bzip2 stream. Compression supports exactly one file.',
    },
    {
      format: 'gz',
      create: 'Yes',
      extract: 'Yes',
      notes: 'Basic gzip stream. Compression supports exactly one file.',
    },
    {
      format: '7z',
      create: 'Yes',
      extract: 'Yes',
      notes: 'Via sevenz-rust. Supports encrypted archive creation and extraction.',
    },
    {
      format: 'rar',
      create: 'No',
      extract: 'No',
      notes: 'Not supported yet. Ziply only lists native archive formats in active support.',
    },
  ]

  return (
    <>
      <article className="panel-card">
        <p className="card-label">Platforms</p>
        <ul className="chip-list">
          {overview.supportedPlatforms.map((platform) => (
            <li className="chip" key={platform}>
              {platform}
            </li>
          ))}
        </ul>
      </article>

      <article className="panel-card">
        <p className="card-label">Supported now</p>
        <ul className="chip-list">
          {overview.activeFormats.map((format) => (
            <li className="chip chip--soft" key={format}>
              {format}
            </li>
          ))}
        </ul>
      </article>

      <article className="panel-card">
        <p className="card-label">Planned later</p>
        <ul className="chip-list">
          {overview.plannedFormats.map((format) => (
            <li className="chip chip--muted" key={format}>
              {format}
            </li>
          ))}
        </ul>
      </article>

      <article className="panel-card">
        <p className="card-label">Native support</p>
        <div className="support-note">
          <strong>Ziply is running native-only.</strong>
          <p>
            All formats listed in Supported now are handled by Ziply itself. Unsupported formats:
            {' '}
            {capabilities.unsupportedFormats.join(', ')}.
          </p>
        </div>
      </article>

      <article className="panel-card panel-card--wide">
        <p className="card-label">Focus areas</p>
        <div className="focus-list">
          {overview.focusAreas.map((item) => (
            <div className="focus-item" key={item}>
              <span className="focus-index" />
              <p>{item}</p>
            </div>
          ))}
        </div>
      </article>

      <article className="panel-card panel-card--wide">
        <p className="card-label">Format matrix</p>
        <div className="format-matrix">
          <div className="format-matrix__row format-matrix__row--header">
            <strong>Format</strong>
            <strong>Compress</strong>
            <strong>Extract</strong>
            <strong>Notes</strong>
          </div>
          {formatSupportRows.map((row) => (
            <div className="format-matrix__row" key={row.format}>
              <strong>{row.format}</strong>
              <span>{row.create}</span>
              <span>{row.extract}</span>
              <span>{row.notes}</span>
            </div>
          ))}
        </div>
      </article>
    </>
  )
}
