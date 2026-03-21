interface EnvironmentPageProps {
  envId: string;
}

export function EnvironmentPage({ envId }: EnvironmentPageProps) {
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold">Environment: {envId}</h1>
    </div>
  );
}
