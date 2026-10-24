import type React from "react";
import { useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { X, Filter } from "lucide-react";
import { SearchBar } from "@/components/searchbar";
import {
  Select,
  SelectTrigger,
  SelectContent,
  SelectItem,
  SelectValue,
} from "@/components/ui/select";

export interface FilterOption {
  value: string | null;
  onChange: (value: string | null) => void;
  options: string[];
  placeholder: string;
  labelFunction?: (value: string) => string;
}

interface FilterSectionProps {
  showFilters: boolean;
  setShowFilters: React.Dispatch<React.SetStateAction<boolean>>;
  clearFilters: () => void;
  isFilterApplied: boolean;
  setSearchTerm: (value: string) => void;
  filterOptions: FilterOption[];
  searchBarName: string;
}

export function FilterSection({
  showFilters,
  setShowFilters,
  clearFilters,
  isFilterApplied,
  setSearchTerm,
  filterOptions,
  searchBarName,
}: FilterSectionProps) {
  const toggleFilters = useCallback(() => {
    setShowFilters((prev) => !prev);
    if (showFilters) clearFilters();
  }, [showFilters, setShowFilters, clearFilters]);

  return (
    <div className="mb-4 flex flex-col md:flex-row gap-4">
      <div className="w-full md:w-166">
        <SearchBar setSearchTerm={setSearchTerm} name={searchBarName} />
      </div>
      <Button
        onClick={toggleFilters}
        className={`w-full md:w-auto ${
          showFilters
            ? "text-foreground outline-solid outline-1 outline-foreground"
            : "text-muted-foreground"
        }`}
        variant="outline"
      >
        <Filter className="w-4 h-4" />
      </Button>
      <AnimatePresence>
        {showFilters && (
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: 0.3, ease: "easeInOut" }}
            className="flex flex-wrap gap-2 overflow-hidden"
          >
            {filterOptions.map((option, index) => (
              <FilterSelect
                key={index}
                value={option.value}
                onChange={option.onChange}
                options={option.options}
                placeholder={option.placeholder}
                labelFunction={option.labelFunction}
              />
            ))}
            <AnimatePresence>
              {isFilterApplied && (
                <motion.div
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  transition={{ duration: 0.3, ease: "easeInOut" }}
                >
                  <Button
                    variant="outline"
                    onClick={clearFilters}
                    className="w-full md:w-auto"
                  >
                    <X className="w-4 h-4 text-destructive" />
                  </Button>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function FilterSelect({
  value,
  onChange,
  options,
  placeholder,
  labelFunction,
}: FilterOption) {
  return (
    <Select value={value || ""} onValueChange={(val) => onChange(val || null)}>
      <SelectTrigger className="w-full md:w-[180px]">
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {options.map((option) => (
          <SelectItem key={option} value={option}>
            {labelFunction ? labelFunction(option) : option}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
