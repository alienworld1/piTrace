import { Navigate, Route, Routes } from "react-router";
import { AppLayout } from "../components/layout/AppLayout";
import { CaseDashboardPage } from "../pages/CaseDashboardPage";
import { CaseFormPage } from "../pages/CaseFormPage";
import { CaseWorkspacePage } from "../pages/CaseWorkspacePage";
import { FileAnalysisPage } from "../pages/FileAnalysisPage";
import { FindingDetailPage } from "../pages/FindingDetailPage";
import { ReportPreviewPage } from "../pages/ReportPreviewPage";

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<CaseDashboardPage />} />
        <Route path="cases/new" element={<CaseFormPage mode="create" />} />
        <Route path="cases/:caseId/edit" element={<CaseFormPage mode="edit" />} />
        <Route path="cases/:caseId" element={<CaseWorkspacePage />} />
        <Route path="cases/:caseId/files/:fileId" element={<FileAnalysisPage />} />
        <Route path="cases/:caseId/findings/:findingId" element={<FindingDetailPage />} />
        <Route path="cases/:caseId/report" element={<ReportPreviewPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
