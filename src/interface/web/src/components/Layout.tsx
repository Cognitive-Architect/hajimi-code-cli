import { ReactNode } from 'react'

interface LayoutProps { children: ReactNode }

/** Layout - main application structure with header and content area */
export function Layout({ children }: LayoutProps) {
  return (
    <div className="layout-container">
      <header className="layout-header">FlexLine WebUI</header>
      <main className="layout-main">{children}</main>
    </div>
  )
}
