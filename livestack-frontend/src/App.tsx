


import { Route, Routes } from 'react-router'

import './App.css'
import { Layout } from './components/layout'
import { ProtectedRoute } from './components/protected-route'
import EscalationPoliciesPage from './pages/escalation-policies'
import IncidentsPage from './pages/incidents'
import LandingPage from './pages/landing'
import MonitorDetailPage from './pages/monitor-detail'
import MonitorsPage from './pages/monitors'
import PublicStatusPage from './pages/public-status'
import SettingsPage from './pages/settings'
import SigninPage from './pages/signin'
import SignupPage from './pages/signup'
import StatusPageDetailPage from './pages/status-page-detail'
import StatusPagesPage from './pages/status-pages'

function App() {
  return (
    <Routes>
      <Route path="/" element={<LandingPage />} />
      <Route path="signin" element={<SigninPage />} />
      <Route path="signup" element={<SignupPage />} />
      <Route path="status/:slug" element={<PublicStatusPage />} />

      <Route element={<ProtectedRoute />}>
        <Route element={<Layout />}>
          <Route path="monitors" element={<MonitorsPage />} />
          <Route path="monitors/:websiteId" element={<MonitorDetailPage />} />
          <Route path="incidents" element={<IncidentsPage />} />
          <Route path="escalation-policies" element={<EscalationPoliciesPage />} />
          <Route path="status-pages" element={<StatusPagesPage />} />
          <Route path="status-pages/:statusPageId" element={<StatusPageDetailPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Route>
    </Routes>
  )
}

export default App