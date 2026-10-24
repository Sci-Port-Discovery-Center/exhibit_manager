import Exhibits from "@/screens/exhibits";
import Parts from "@/screens/parts";
import Jotforms from "@/screens/jotforms";
import Dev from "@/screens/dev"

import { Atom, Bolt, NotepadText, ImagePlay } from "lucide-react";

export interface RouteConfig {
  url: string;
  screen: React.ComponentType;
  isActive: boolean;
  title: string;
  sidebar: boolean;
  icon?: React.ComponentType;
  items?: {
    title: string;
    url: string;
    isActive: boolean;
  }[];
}

export const routes: RouteConfig[] = [
  {
    url: "/jotforms",
    screen: Jotforms,
    title: "Jotforms",
    icon: NotepadText,
    isActive: false,
    sidebar: true,
  },
  {
    url: "/exhibits",
    screen: Exhibits,
    title: "Exhibits",
    icon: Atom,
    isActive: true,
    sidebar: true,
  },
  {
    url: "/parts",
    screen: Parts,
    title: "Parts",
    icon: Bolt,
    isActive: false,
    sidebar: true,
  },
  /* 
  {
    url: "/issues",
    screen: Issues,
    title: "Issues",
    icon: CircleDot,
    isActive: false,
    sidebar: true,
  },
  {
    url: "/jotforms",
    screen: Jotforms,
    title: "Jotforms",
    icon: NotebookPen,
    isActive: false,
    sidebar: true,
  },
  */
];
