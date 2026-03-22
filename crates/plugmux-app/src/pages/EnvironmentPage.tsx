// TODO(Task 11): Rewrite with new data model
interface EnvironmentPageProps {
  envId: string;
}

export function EnvironmentPage({ envId }: EnvironmentPageProps) {
  return (
    <div className="p-6">
      <p className="text-sm text-muted-foreground">
        Environment: {envId}
      </p>
    </div>
  );
}
