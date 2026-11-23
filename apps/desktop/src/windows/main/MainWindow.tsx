import { useEffect, useState } from "react";
import { getAppInfo } from "../../features/app/api";
import { FocusScreen } from "../../features/focus/FocusScreen";
import { GoalsScreen } from "../../features/plan/GoalsScreen";
import { PlanScreen } from "../../features/plan/PlanScreen";
import { usePlanStore } from "../../features/plan/store";
import { Sidebar, type Screen } from "./Sidebar";

export function MainWindow() {
  const [version, setVersion] = useState<string | null>(null);
  const [screen, setScreen] = useState<Screen>("focus");
  const [goalId, setGoalId] = useState<string | null>(null);
  const plan = usePlanStore((s) => s.view);
  const connectPlan = usePlanStore((s) => s.connect);

  useEffect(() => {
    getAppInfo()
      .then((info) => setVersion(info.version))
      .catch(() => setVersion(null));
  }, []);
  useEffect(() => connectPlan(), [connectPlan]);

  const navigate = (to: Screen, goal?: string) => {
    setScreen(to);
    if (goal) setGoalId(goal);
  };

  return (
    <div className="flex h-full bg-[radial-gradient(1100px_700px_at_10%_6%,rgba(242,181,68,0.14),transparent_60%),radial-gradient(1000px_700px_at_94%_96%,rgba(108,182,255,0.10),transparent_60%)]">
      <Sidebar
        version={version}
        screen={screen}
        onNavigate={navigate}
        goals={plan?.goals ?? []}
        projects={plan?.projects ?? []}
      />
      <main className="flex min-w-0 flex-1 flex-col">
        {screen === "focus" && <FocusScreen onOpenPlan={() => navigate("plan")} />}
        {screen === "plan" && <PlanScreen onOpenGoals={(id) => navigate("goals", id)} />}
        {screen === "goals" && (
          <GoalsScreen
            selectedId={goalId}
            onSelect={setGoalId}
            onOpenPlan={() => navigate("plan")}
          />
        )}
      </main>
    </div>
  );
}
