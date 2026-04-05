import { HistoryIcon, LogoIcon, SettingsIcon, WorkspaceIcon } from './AppIcons'

export type AppSection = 'workspace' | 'history' | 'settings'

interface AppSidebarProps {
  activeSection: AppSection
  onSelectSection: (section: AppSection) => void
}

const sidebarItems: Array<{
  id: AppSection
  label: string
  Icon: typeof WorkspaceIcon
}> = [
  { id: 'workspace', label: 'Workspace', Icon: WorkspaceIcon },
  { id: 'history', label: 'History', Icon: HistoryIcon },
  { id: 'settings', label: 'Settings', Icon: SettingsIcon },
]

export function AppSidebar({ activeSection, onSelectSection }: AppSidebarProps) {
  return (
    <aside className="app-sidebar">
      <div className="app-sidebar__brand">
        <div className="app-sidebar__logo">
          <LogoIcon />
        </div>
        <div>
          <strong>Ziply</strong>
        </div>
      </div>

      <nav className="app-sidebar__nav" aria-label="Primary">
        {sidebarItems.map(({ id, label, Icon }) => (
          <button
            className={`app-sidebar__nav-item ${activeSection === id ? 'app-sidebar__nav-item--active' : ''}`}
            key={id}
            onClick={() => {
              onSelectSection(id)
            }}
            type="button"
          >
            <Icon />
            <span>{label}</span>
          </button>
        ))}
      </nav>

      <div className="app-sidebar__footer">v0.1.0 • Native Core</div>
    </aside>
  )
}
