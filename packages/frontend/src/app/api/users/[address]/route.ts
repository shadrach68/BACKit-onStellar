import { NextRequest, NextResponse } from 'next/server'
import { getUserProfile } from '@/lib/mockDb'

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ address: string }> }
) {
  const { address } = await params

  const userProfile = getUserProfile(address)

  return NextResponse.json(userProfile)
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ address: string }> }
) {
  const { address } = await params
  const url = new URL(request.url)
  const action = url.pathname.split('/').pop()

  const userProfile = getUserProfile(address)

  if (action === 'follow') {
    if (userProfile) {
      userProfile.user.followers += 1
      userProfile.user.isFollowing = true
    }
    return NextResponse.json({ success: true })
  }

  if (action === 'unfollow') {
    if (userProfile) {
      userProfile.user.followers -= 1
      userProfile.user.isFollowing = false
    }
    return NextResponse.json({ success: true })
  }

  return NextResponse.json(
    { error: 'Invalid action' },
    { status: 400 }
  )
}

