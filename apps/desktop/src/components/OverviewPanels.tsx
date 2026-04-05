import type { AppOverview, ArchiveCapabilities } from '../app/types'

interface OverviewPanelsProps {
  overview: AppOverview
  capabilities: ArchiveCapabilities
}

export function OverviewPanels({ overview, capabilities }: OverviewPanelsProps) {
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
        <p className="card-label">Tool bridge</p>
        <div className="support-note">
          <strong>
            {capabilities.rarExtractionAvailable
              ? 'RAR extraction is enabled.'
              : 'RAR extraction is currently unavailable.'}
          </strong>
          <p>
            {capabilities.rarExtractionAvailable
              ? `Ziply found ${capabilities.rarExtractorLabel} on this machine and can use it for .rar extraction.`
              : 'Install unar, 7zz, 7z, or unrar to unlock extract-only support for .rar archives.'}
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
    </>
  )
}
