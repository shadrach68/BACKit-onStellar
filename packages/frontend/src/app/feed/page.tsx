"use client";

import { useState } from "react";
import FeedTabs from "@/components/FeedTabs";
import { CallCardSkeleton } from "@/components/CardCallSkeleton";
import { EmptyState } from "@/components/EmptyState";
import CallCard from "@/components/CallCard";
import { useFeed } from "@/hooks/useFeed";
import FilterBar from "@/components/FilterBar";
import { useInfiniteScroll } from "@/hooks/useInfiniteScroll";
import Link from "next/link";
import { ArrowUp } from "lucide-react";

export default function FeedPage() {
  const [tab, setTab] = useState<"for-you" | "following">("for-you");
  const [filters, setFilters] = useState<{ status: string | null }>({ status: null });
  
  const { items, loading, loadingMore, hasMore, loadMore } = useFeed(tab, filters);

  const cacheKey = `${tab}-${filters.status || 'all'}`;
  const { triggerRef, showBackToTop, scrollToTop } = useInfiniteScroll({
    loadMore,
    hasMore,
    loadingMore,
    items,
    cacheKey,
  });

  const handleFilterChange = (newFilters: { status: string | null }) => {
    setFilters(newFilters);
  };

  return (
    <main className="max-w-2xl mx-auto p-4 relative min-h-screen pb-16">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Prediction Feed</h1>
        <p className="text-gray-600">Explore trending predictions and stake on outcomes</p>
      </div>
      
      <FeedTabs active={tab} onChange={setTab} />
      <FilterBar onFilterChange={handleFilterChange} />

      {loading && (
        <div className="space-y-4 mt-4">
          {[...Array(6)].map((_, i) => (
            <CallCardSkeleton key={i} />
          ))}
        </div>
      )}

      {!loading && items.length === 0 && (
        <EmptyState
          text={
            tab === "for-you"
              ? filters.status
                ? `No ${filters.status.toLowerCase()} calls found in "For You" feed.`
                : "No trending calls yet. Check back later for new predictions!"
              : filters.status
                ? `No ${filters.status.toLowerCase()} calls from users you follow.`
                : "Follow users to see their calls."
          }
        />
      )}

      <div className="space-y-4 mt-4">
        {items.map((call) => (
          <Link key={call.id} href={`/calls/${call.id}`} className="block">
            <CallCard call={call} />
          </Link>
        ))}
      </div>

      {/* Infinite Scroll Trigger */}
      {hasMore && !loading && (
        <div ref={triggerRef} className="flex justify-center py-6">
          <svg className="animate-spin h-8 w-8 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
      )}

      {/* End of results message */}
      {!hasMore && items.length > 0 && (
        <div className="text-center text-gray-500 py-8 font-medium">
          No more markets
        </div>
      )}

      {/* Back to Top button */}
      {showBackToTop && (
        <button
          onClick={scrollToTop}
          className="fixed bottom-6 right-6 p-3 bg-indigo-600 hover:bg-indigo-700 text-white rounded-full shadow-lg transition-all duration-300 transform hover:scale-110 z-50 flex items-center justify-center"
          aria-label="Back to top"
        >
          <ArrowUp className="w-5 h-5" />
        </button>
      )}
    </main>
  );
}
