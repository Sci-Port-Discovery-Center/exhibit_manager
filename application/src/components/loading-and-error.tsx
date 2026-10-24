import { Loader2 } from "lucide-react";

export function Loading() {
  return (
    <div className="h-screen flex justify-center items-center pb-60">
      <Loader2 className="animate-spin h-8 w-8" />
    </div>
  );
}

export function Error({ name }: { name: string }) {
  return (
    <div className="h-screen flex justify-center items-center pb-60">
      Error fetching Data
    </div>
  );
}

export function NoData({ name }: { name: string }) {
  return (
    <div className="h-screen flex justify-center items-center pb-60">
      No Data found
    </div>
  );
}
