import type { MetadataField } from "../../types/forensics";
import { Badge } from "../ui/Badge";

interface MetadataTableProps {
  fields: MetadataField[];
}

export function MetadataTable({ fields }: MetadataTableProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <h2 className="font-display text-xl font-semibold text-ink">Mapped field details</h2>
      <p className="mt-1 text-sm text-muted">Original ExifTool groups behind the readable fields above.</p>
      <div className="mt-5 overflow-hidden rounded-lg border border-line">
        {fields.length === 0 ? (
          <p className="bg-surface px-4 py-5 text-sm text-muted">No mapped metadata fields are available yet.</p>
        ) : (
          <table className="w-full border-collapse text-left text-sm">
            <thead className="bg-panel-high text-xs uppercase tracking-[0.05em] text-primary-soft">
              <tr>
                <th className="px-4 py-3 font-semibold">Group</th>
                <th className="px-4 py-3 font-semibold">Key</th>
                <th className="px-4 py-3 font-semibold">Value</th>
                <th className="px-4 py-3 font-semibold">Source</th>
              </tr>
            </thead>
            <tbody>
              {fields.map((field) => (
                <tr className="border-t border-line bg-surface" key={field.id}>
                  <td className="px-4 py-3 technical text-xs text-cyan">{field.group}</td>
                  <td className="px-4 py-3 text-ink">{field.key}</td>
                  <td className="px-4 py-3 text-muted">{field.value}</td>
                  <td className="px-4 py-3">
                    <Badge tone="neutral">{field.source}</Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </section>
  );
}
