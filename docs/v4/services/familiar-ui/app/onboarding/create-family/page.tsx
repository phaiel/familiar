"use client"

import React, { useState, useEffect } from "react"
import { useRouter } from "next/navigation"
import { ArrowRight, Loader2, Users, Sparkles } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useAuthStore } from "@/stores/auth-store"

export default function CreateFamilyPage() {
  const router = useRouter()
  
  const { 
    user,
    isAuthenticated,
    needsFamily,
    createFamily,
    isLoading,
    error,
    clearError,
  } = useAuthStore()

  const [familyName, setFamilyName] = useState("")
  const [creating, setCreating] = useState(false)

  // Redirect if not authenticated or already has family
  useEffect(() => {
    if (!isAuthenticated) {
      router.push("/onboarding")
    } else if (!needsFamily) {
      router.push("/")
    }
  }, [isAuthenticated, needsFamily, router])

  // Pre-fill family name based on user's name
  useEffect(() => {
    if (user?.name && !familyName) {
      const lastName = user.name.split(" ").pop() || user.name
      setFamilyName(`The ${lastName} Family`)
    }
  }, [user])

  const handleCreateFamily = async (e: React.FormEvent) => {
    e.preventDefault()
    clearError()
    setCreating(true)
    
    console.log("[CreateFamily] Starting family creation:", familyName)
    
    try {
      const result = await createFamily(familyName)
      console.log("[CreateFamily] Success:", result)
      router.push("/")
    } catch (err) {
      console.error("[CreateFamily] Error:", err)
      // Error is handled by store
    } finally {
      setCreating(false)
    }
  }

  if (!isAuthenticated) {
    return null
  }

  return (
    <Card>
      <CardHeader className="text-center">
        <div className="mx-auto w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mb-2">
          <Users className="h-6 w-6 text-primary" />
        </div>
        <CardTitle>Create your family</CardTitle>
        <CardDescription>
          Give your family a name. You can invite others to join later.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleCreateFamily} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="family-name">Family Name</Label>
            <Input
              id="family-name"
              placeholder="The Smith Family"
              value={familyName}
              onChange={(e) => setFamilyName(e.target.value)}
              required
            />
            <p className="text-xs text-muted-foreground">
              This is how your family will appear to members
            </p>
          </div>

          {error && (
            <div className="rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
              {error}
            </div>
          )}

          <Button type="submit" className="w-full" disabled={creating || !familyName.trim()}>
            {creating ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <>
                Create Family
                <ArrowRight className="ml-2 h-4 w-4" />
              </>
            )}
          </Button>
        </form>
      </CardContent>
      <CardFooter>
        <div className="w-full rounded-lg bg-muted/50 p-4 space-y-2">
          <div className="flex items-center gap-2 text-sm font-medium">
            <Sparkles className="h-4 w-4 text-primary" />
            What happens next?
          </div>
          <ul className="text-xs text-muted-foreground space-y-1 ml-6 list-disc">
            <li>You&apos;ll be the admin of your family</li>
            <li>A shared family channel will be created</li>
            <li>You can invite family members with a code</li>
            <li>Mention @nona in the family chat to talk to AI</li>
          </ul>
        </div>
      </CardFooter>
    </Card>
  )
}

