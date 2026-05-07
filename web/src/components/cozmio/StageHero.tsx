import Image from "next/image";
import type { ReactNode } from "react";

type StageHeroProps = {
  eyebrow: ReactNode;
  title: ReactNode;
  body: ReactNode;
  visualSrc: string;
  visualAlt: string;
  actions?: ReactNode;
  pills?: ReactNode;
  overlay?: ReactNode;
  className?: string;
  visualClassName?: string;
};

export function StageHero({
  eyebrow,
  title,
  body,
  visualSrc,
  visualAlt,
  actions,
  pills,
  overlay,
  className = "",
  visualClassName = "",
}: StageHeroProps) {
  return (
    <section className={`coz-stage-shell relative pt-6 md:pt-8 ${className}`}>
      <div className="coz-stage-hero">
        <div className="coz-stage-light" />
        <Image
          src={visualSrc}
          alt={visualAlt}
          width={1280}
          height={820}
          priority
          className={`coz-stage-visual ${visualClassName}`}
        />
        <div className="coz-stage-copy">
          {eyebrow}
          <h1>{title}</h1>
          <p>{body}</p>
          {pills ? <div className="coz-stage-pills">{pills}</div> : null}
          {actions ? <div className="coz-stage-actions">{actions}</div> : null}
        </div>
        {overlay ? <div className="coz-stage-overlay">{overlay}</div> : null}
      </div>
    </section>
  );
}
