import { Lock, ExternalLink } from "lucide-react";
import { Space, Tier, gumroadUrlForTier, tierLabel } from "../../lib/spaces";

type Props = {
  space: Space;
  userTier: Tier;
};

export function LockedSpace({ space, userTier: _userTier }: Props) {
  const url = gumroadUrlForTier(space.tier);
  return (
    <section className="h-full flex items-center justify-center p-8">
      <div className="max-w-md text-center space-y-5">
        <div className="inline-flex w-14 h-14 rounded-2xl bg-zinc-900/60 border border-zinc-800 items-center justify-center mx-auto">
          <Lock size={22} className="text-zinc-400" />
        </div>
        <h1 className="text-2xl font-semibold tracking-tight">{space.label}</h1>
        <p className="text-sm text-zinc-400 leading-relaxed">
          {space.description}
        </p>
        <p className="text-xs text-zinc-500">
          Cet espace est inclus dans le tier{" "}
          <span className="text-andrea-300">{tierLabel(space.tier)}</span>.
          Vous pouvez payer la différence depuis votre tier actuel — vos
          données existantes ne seront pas perdues.
        </p>
        <a
          href={url}
          target="_blank"
          rel="noreferrer noopener"
          className="inline-flex items-center gap-1.5 rounded-md bg-andrea-600 hover:bg-andrea-500 px-4 py-2 text-sm font-medium text-white transition-colors"
        >
          Découvrir le tier {tierLabel(space.tier)}
          <ExternalLink size={14} />
        </a>
      </div>
    </section>
  );
}
