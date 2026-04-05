import type { SVGProps } from 'react'

type IconProps = SVGProps<SVGSVGElement>

function BaseIcon(props: IconProps) {
  return (
    <svg
      aria-hidden="true"
      fill="none"
      height="20"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.8"
      viewBox="0 0 24 24"
      width="20"
      {...props}
    />
  )
}

export function LogoIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <rect height="16" rx="3.5" width="16" x="4" y="4" />
      <path d="M8 8h8" />
      <path d="M8 12h8" />
      <path d="M10 16h4" />
    </BaseIcon>
  )
}

export function WorkspaceIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <rect height="6" rx="1.5" width="6" x="4" y="4" />
      <rect height="6" rx="1.5" width="6" x="14" y="4" />
      <rect height="6" rx="1.5" width="6" x="4" y="14" />
      <rect height="6" rx="1.5" width="6" x="14" y="14" />
    </BaseIcon>
  )
}

export function HistoryIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M3 12a9 9 0 1 0 3-6.7" />
      <path d="M3 4v5h5" />
      <path d="M12 7v5l3 2" />
    </BaseIcon>
  )
}

export function SettingsIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M10.4 2.8 9.7 5a7.4 7.4 0 0 0-1.9.8L5.7 4.8 4 6.5l1 2.1a7.4 7.4 0 0 0-.8 1.9L2 11.2v2.6l2.2.7c.1.7.4 1.3.8 1.9L4 18.5l1.7 1.7 2.1-1a7.4 7.4 0 0 0 1.9.8l.7 2.2h2.6l.7-2.2a7.4 7.4 0 0 0 1.9-.8l2.1 1 1.7-1.7-1-2.1c.4-.6.7-1.2.8-1.9l2.2-.7v-2.6l-2.2-.7a7.4 7.4 0 0 0-.8-1.9l1-2.1-1.7-1.7-2.1 1a7.4 7.4 0 0 0-1.9-.8l-.7-2.2z" />
      <circle cx="12" cy="12" r="3.25" />
    </BaseIcon>
  )
}

export function DropIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M8 17a4 4 0 1 1 .8-7.9A5.5 5.5 0 0 1 19 11a3.5 3.5 0 0 1-.5 7H8Z" />
      <path d="m12 11 0 7" />
      <path d="m9.5 13.5 2.5-2.5 2.5 2.5" />
    </BaseIcon>
  )
}

export function ArchiveIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M6 5h12l-1 14H7L6 5Z" />
      <path d="M9 5V3h6v2" />
      <path d="M9 10h6" />
      <path d="M10 14h4" />
    </BaseIcon>
  )
}

export function CompressIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M12 4v10" />
      <path d="m8.5 10.5 3.5 3.5 3.5-3.5" />
      <rect height="5" rx="2" width="14" x="5" y="15" />
    </BaseIcon>
  )
}

export function ExtractIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <rect height="5" rx="2" width="14" x="5" y="4" />
      <path d="M12 9v10" />
      <path d="m8.5 15.5 3.5 3.5 3.5-3.5" />
    </BaseIcon>
  )
}

export function CheckIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="12" cy="12" r="8.5" />
      <path d="m8.5 12.5 2.2 2.2 4.8-5.2" />
    </BaseIcon>
  )
}

export function ErrorIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="12" cy="12" r="8.5" />
      <path d="m9.5 9.5 5 5" />
      <path d="m14.5 9.5-5 5" />
    </BaseIcon>
  )
}

export function ClockIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <circle cx="12" cy="12" r="8.5" />
      <path d="M12 8v4.5l3 1.8" />
    </BaseIcon>
  )
}

export function FolderIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M3.5 7.5h6l2 2h9v7.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z" />
      <path d="M3.5 7.5V7a2 2 0 0 1 2-2h4l2 2" />
    </BaseIcon>
  )
}

export function ShieldIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M12 3 6 5.5v5.8c0 4.1 2.4 7.8 6 9.7 3.6-1.9 6-5.6 6-9.7V5.5z" />
      <path d="m9.5 12 1.7 1.8 3.3-3.8" />
    </BaseIcon>
  )
}

export function SlidersIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M4 6h8" />
      <path d="M16 6h4" />
      <path d="M10 6a2 2 0 1 0 4 0 2 2 0 0 0-4 0Z" />
      <path d="M4 12h4" />
      <path d="M12 12h8" />
      <path d="M8 12a2 2 0 1 0 4 0 2 2 0 0 0-4 0Z" />
      <path d="M4 18h10" />
      <path d="M18 18h2" />
      <path d="M14 18a2 2 0 1 0 4 0 2 2 0 0 0-4 0Z" />
    </BaseIcon>
  )
}

export function TrashIcon(props: IconProps) {
  return (
    <BaseIcon {...props}>
      <path d="M4 7h16" />
      <path d="M9 7V5h6v2" />
      <path d="m6 7 1 12h10l1-12" />
      <path d="M10 11v5" />
      <path d="M14 11v5" />
    </BaseIcon>
  )
}
